<!-- markdownlint-disable MD033 MD041 -->

# 隐藏标记与特殊标记（Hidden / Special Flags）

> [← 设计文档索引](./README.md) · 相关：[BinaryMD5 归档导出](./binarymd5-archive-export.md) · [打点审批工作流](./punctuate-workflow.md)

本文解释原神地图后端三套**正交**的可见性 / 过滤标记：`hidden_flag`（谁能看）、
`special_flag`（查不查得出来）、`del_flag`（软删除）。它们名字相近、都挂在点位 /
物品 / 地区实体上，但语义完全不同，混用会把测试服数据漏给正式服玩家、或把已删点位
重新暴露给全服。源码位置：

- 枚举定义：`packages/utils/src/types/common.rs`（`HiddenFlag`）
- 实体字段：`marker.rs` / `item.rs` / `area.rs`（`hidden_flag` / `special_flag`）
- 过滤逻辑：`packages/functions/src/functions/api/item.rs`（`special_flag` 位掩码）
- 归档分片：`packages/functions/src/functions/api/marker_doc.rs` / `item_doc.rs`

## 1. 为什么需要这么多套标记

原神地图不是「所有数据对所有人可见」这么简单，它至少要处理三种现实需求：

- **防剧透**：玩家还没解锁稻妻，地图上就不该出现稻妻的雷神瞳位置；某些彩蛋类

点位（比如隐藏成就触发点）只该对主动选择「看彩蛋」的玩家显示。这是**可见性**
问题。

- **测试服 / 内鬼数据隔离**：新版本上线前，编辑团队要提前在地图里录入下个版本

（如枫丹、纳塔）的点位，这些数据绝对不能漏给正式服玩家（会剧透 + 违规）。这
是**数据隔离**问题。

- **物品查询过滤**：地图右上角的物品筛选面板里，有些物品是「特殊物品」（比如活动

限定、前台默认不显示的），玩家要勾选「显示特殊物品」才查得出来。这是**查询过滤**
问题。

- **软删除**：点位下线后不能直接从库里抹掉（要保留审计轨迹、防止误删），但又不能

再被任何查询返回。这是**生命周期**问题。

`hidden_flag`、`special_flag`、`del_flag` 三套标记各自服务其中一类，且**互不耦合**：
一个点位的可见性、是否可查、是否已删，是三个独立的维度。下面分别展开。

## 2. hidden_flag：数据级可见性过滤

`HiddenFlag`（`packages/utils/src/types/common.rs:9`）是社区地图特有的**数据级可见性**
枚举，存为 i32：

| 枚举成员 | 数值 | 含义 | 谁能看 |
| --- | --- | --- | --- |
| `Visible` | `0` | 正式数据 | 所有玩家 |
| `Hidden` | `1` | 隐藏 | 仅内鬼 / 后台 |
| `Spy` | `2` | 测试服 | 测试服玩家 |
| `Suprise` | `3` | 彩蛋 | 主动开启彩蛋的玩家 |

注意：这是**数据级的过滤**，不是接口级的权限校验。它解决的不是「这个用户能不能调
这个 API」，而是「这个用户能在地图上看到哪些点位」。前端通过请求头 `userDataLevel`
（一个**位掩码**）声明自己有权看到哪些层级——比如普通玩家传的掩码里只置了 `Visible`
位，内鬼客户端会同时置 `Hidden` / `Spy` 位。

这套标记的两个主要用途：

- **防剧透 / 彩蛋隔离**：玩家尚未解锁的区域里的秘密（比如须弥某些梦之树宝箱）、

藏得很深的彩蛋点位，标成 `Suprise`，默认不显示，玩家主动勾选「显示彩蛋」才拉取。
这样既保留了「全收集」工具的完整性，又不破坏探索乐趣。

- **测试数据隔离**：下个版本的新地区（枫丹廷、纳塔部落等）点位在录入阶段标成

`Spy`，正式服前端拉不到；版本上线当天把对应点位的 `hidden_flag` 改回 `Visible`，
全服即时可见。这避免了「上线日临时录数据」的慌乱。

### 2.1 下沉到归档层

`hidden_flag` 不只是查询时过滤，而是**在 BinaryMD5 归档分片时就作为第一级分组键**
（见 [归档导出文档·第 3.1 节](./binarymd5-archive-export.md)）。`marker_doc`
和 `item_doc` 都按 `hidden_flag` 分组各自压缩成独立的 gzip blob、各自一个 MD5。这意味着
普通玩家客户端**根本不会收到** `Hidden`/`Spy`/`Suprise` 组的 MD5——不是查询时过滤掉，
而是数据分片压根不下发。这是比「查询过滤」更强的隔离：哪怕前端有 bug 漏过滤，
数据也没传到客户端。

### 2.2 实体字段位置

`hidden_flag` 出现在所有内容域实体上，且晋升时自动继承：

- `marker.hidden_flag`（点位可见性）
- `item.hidden_flag`（物品可见性）
- `area.hidden_flag`（地区可见性）
- `marker_punctuate.hidden_flag`——打点建议也带这个字段，`do_pass` 晋升时原样带进

正式 `marker`（见 [打点工作流](./punctuate-workflow.md#4-method_type三种晋升路径)）。
所以一个点位上线后就拥有提交者设定的可见性。

字段都加了 `#[sea_orm(indexed)]`，因为归档分组查询要按它过滤。

## 3. special_flag：物品/地区查询的位掩码过滤

`special_flag` 是一个 **i32 位掩码**（item 上是 `Option<i32>`，area 上是 `i32`），
和 `hidden_flag` 完全是两回事：它不控制「谁能看」，而是控制**物品筛选面板里查不查
得出来**。

### 3.1 语义：低位第一位 = 前台是否显示

`item.rs` 模型注释写明（`packages/database/src/models/item/item.rs:54`）：

> 特殊物品标记
> 低位第一位: 前台是否显示

也就是说第 0 位（值 1）表示「这是前台默认不显示的特殊物品」。地图客户端右上角的
物品筛选面板里，默认只列出 `special_flag = 0`（无特殊标记）的普通物品；玩家勾选
「显示特殊物品」后，才会把含特殊位的物品也查出来。典型场景：活动限定物品、调试用
物品、稀有度极高的隐藏收集品。

### 3.2 查询逻辑：Java `selectPageItemByCondition` 的对齐

`item.rs::do_get_list`（`packages/functions/src/functions/api/item.rs:68`）实现了这个
位掩码过滤，严格对齐 Java 的 `selectPageItemByCondition` 自定义查询：

```rust
if let Some(sf) = payload.special_flag {
    // Java parity: special_flag is a bit-mask. param == 0 means "no special
    // flag set" (filter special_flag = 0); param > 0 means "has any of these
    // bits" (filter (special_flag & param) != 0).
    let sf = sf as i32;
    if sf == 0 {
        query = query.filter(item_model::Column::SpecialFlag.eq(0));
    } else {
        query = query.filter(Expr::col(item_model::Column::SpecialFlag).bit_and(sf).ne(0));
    }
}
```

两种分支：

- **客户端传 `special_flag = 0`**（默认）：精确过滤 `special_flag = 0` 的物品，即

「无任何特殊标记的普通物品」。这是物品筛选面板的默认行为。

- **客户端传 `special_flag > 0`**（某位被置 1）：过滤 `(special_flag & param) != 0`，

即「至少含有客户端指定的某一位特殊标记」的物品。位与运算支持未来扩展更多位
（第 1 位、第 2 位……各代表一种特殊属性），客户端按位组合查询。

这个过滤和分页、地区筛选、名称模糊、类型筛选叠加在一起（`do_get_list` 同时支持
`area_id_list` / `name` / `type_id_list`），共同构成物品查询 UI 的后端。

### 3.3 area 上的 special_flag

地区表（`area.rs:46`）也有 `special_flag`，语义同样是「低位第一位：前台是否显示」。
用于在地区选择树里隐藏某些调试地区或活动地区。

## 4. 三套标记的正交关系

这三个标记经常被新人搞混，但它们是**完全正交**的三个维度：

| 标记 | 类型 | 控制的维度 | 典型问题 |
| --- | --- | --- | --- |
| `hidden_flag` | 枚举（0~3） | **谁能看**（数据级可见性） | 「测试服玩家才能看到这个点位」 |
| `special_flag` | i32 位掩码 | **查不查得出来**（查询过滤） | 「勾选特殊物品才筛得出来」 |
| `del_flag` | bool | **是否已删**（生命周期） | 「这个点位下线了，任何查询都不能返回」 |

一个点位可以同时：已经被软删（`del_flag=true`）、原本是测试服数据
（`hidden_flag=Spy`）、且是特殊物品（`special_flag=1`）。三者互不影响。

### 4.1 与软删除 del_flag 的正交关系

`del_flag` 是**生命周期标记**，由 `SafeEntityTrait`（见
[架构概览](../guides/architecture.md#safeentitytrait-模式)）统一管理。它的规则
是：**被软删的点位，不论其 `hidden_flag` / `special_flag` 是什么，都一律被排除**。

这是因为 `SafeEntityTrait::find_safety()` 在所有查询的根部都加了 `del_flag = false`
过滤——`marker_doc.rs` / `item_doc.rs` 里查全量数据用的就是 `Entity::find_safety().all()`，
所以归档分片天然不含软删点位；`do_get_list` 等业务查询也都走 `find_safety`，软删物品
查不到。换句话说：

- `hidden_flag` / `special_flag` 是**业务过滤**（在不同场景下选择性地返回数据）；
- `del_flag` 是**存在性过滤**（数据「物理上」还在库里，但「逻辑上」已经不存在了）。

两者叠加的语义是：「先排除已删的，再在剩下的里按可见性 / 特殊性过滤」。哪怕一个点位
的 `hidden_flag=Visible`（对所有人可见），一旦 `del_flag=true`，它就从所有查询和归档
里消失——这正是软删除该有的行为。

## 5. 常见误用与规避

- **把 `hidden_flag` 当权限校验**：`hidden_flag` 是数据级过滤，不是安全边界。真正

的「谁能调测试服 API」要在路由层用 `ExtractAuthInfo` + 角色中间件做。`hidden_flag`
只是保证「即便前端漏过滤，数据也不下发」（通过归档分片），不替代鉴权。

- **把 `special_flag` 当 `hidden_flag` 用**：`special_flag=1` 的物品在默认物品筛选里

查不到，但它的点位（如果 `hidden_flag=Visible`）仍然会出现在地图上和归档里。
想真正隐藏一个点位，要改 `hidden_flag`。

- **晋升时漏带 `hidden_flag`**：打点建议晋升为正式点位时，`do_pass` 必须把

`punctuate.hidden_flag` 原样写进 `marker.hidden_flag`（`punctuate_audit.rs:124`）。
如果漏掉，提交者标成 `Suprise` 的彩蛋点会变成全服可见，剧透就漏出去了。

## 6. 与 Java 实现的对齐

| Java（`java-genshin-map-cloud`） | Rust |
| --- | --- |
| `HiddenFlag` 枚举（0~3） | `HiddenFlag` 枚举（`Visible/Hidden/Spy/Suprise`） |
| `userDataLevel` 请求头位掩码 | 由前端按位组合，后端按 flag 分组归档 |
| `selectPageItemByCondition`（special_flag 位掩码） | `item::do_get_list`（`bit_and` + `eq(0)` 两分支） |
| `BaseEntity.del_flag`（MyBatis-Plus 逻辑删除） | `SafeEntityTrait::find_safety`（`del_flag=false` 过滤） |

注意 Rust 枚举里彩蛋成员拼写是 `Suprise`（少一个 `r`，`packages/utils/src/types/common.rs:21`），
这是为了和 Java 侧的历史拼写保持字节级一致（避免反序列化对不上），属有意为之，
不是笔误——改拼写会破坏与 Java 数据库存量数据的兼容。
