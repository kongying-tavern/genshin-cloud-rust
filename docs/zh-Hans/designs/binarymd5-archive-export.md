<!-- markdownlint-disable MD033 MD041 -->

# BinaryMD5 归档导出管线

> [← 设计文档索引](./README.md) · 相关：[隐藏/特殊标记](./hidden-and-special-flags.md)

本文解释 Rust 后端的 `*_doc` 端点家族——为什么客户端冷启动要下载一大坨压缩
字节流、为什么要按 MD5 寻址、为什么 marker 的正式组要按 `id / 3000` 切页，以及
这套管线在 Rust 侧当前的对齐状态。源码位置：

- 公共压缩工具：`packages/functions/src/functions/api/binary_doc.rs`
- marker 归档：`packages/functions/src/functions/api/marker_doc.rs`
- 物品归档：`packages/functions/src/functions/api/item_doc.rs`
- 点位关联归档：`packages/functions/src/functions/api/marker_link_doc.rs`
- 图标 / 标签归档：`packages/functions/src/functions/api/icon_doc.rs` / `tag_doc.rs`

## 1. 为什么存在：冷启动的全量拉取问题

打开空荧酒馆地图客户端的瞬间，前端要拿到**全服所有点位、所有物品、所有点位关联**
才能渲染地图。这不是几十条数据——一个成熟版本的原神地图，光 marker（神瞳、宝箱、
奇馈宝箱、采集物、地笼、解谜机关……）就有数千上万个 POI，遍布蒙德、璃月、稻妻、
须弥、枫丹、纳塔、至冬等所有地区。逐条 JSON-over-HTTP 拉取（哪怕每页 100 条）
意味着几十上百个串行请求，首屏会卡到不可用。

所以后端提供的是 **GZIP 压缩的整体 JSON blob**：把一整组实体序列化成 JSON、压成
gzip、以压缩后字节的 MD5 作为键来寻址。客户端只需先拉一份「MD5 清单」（很小的
JSON 数组），比对本地已缓存的 MD5，**只下载发生变化的页**。这在三个层面受益：

- **压缩**：点位数据高度重复（坐标格式、物品 ID、图标 ID 都是小整数或短串），gzip
  后体积通常能压到原 JSON 的 10%~20%，移动端流量友好；
- **增量**：MD5 内容寻址让客户端天然支持「只取变更页」——绝大多数冷启动其实是
  热启动，本地 MD5 命中后一个字节都不用下；
- **幂等**：相同数据算出的 MD5 永远一致，CDN / 对象存储可直接按 MD5 键长缓存。

这对应 Java 侧的 `CompressUtils` + `DigestUtils` + Caffeine 缓存管线，是社区地图
工具冷启动性能的关键基础设施。

## 2. 压缩管线：序列化 → GZIP → MD5

核心三步在 `binary_doc.rs::serialize_compress_md5` 里，和 Java `CompressUtils`
严格对齐：

```rust
// 1. 序列化为 JSON（UTF-8 字节）
let json = serde_json::to_vec(data)?;
// 2. GZIP 压缩
let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
encoder.write_all(&json)?;
let compressed = encoder.finish()?;
// 3. 对【压缩后字节】算 MD5（小写 hex，32 字符）
let digest = md5::compute(&compressed);
let md5_hex = format!("{:x}", digest);
```

一个常被忽视但很关键的点：**MD5 是对压缩后字节算的，不是对原始 JSON 算的**。
这意味着 MD5 同时是「内容指纹」和「压缩产物」的键——客户端拿到 `list_page_bin/{md5}`
返回的字节流，可以直接 GZIP 解压得到原始 JSON，无需额外元数据。

## 3. 切页策略：按 hidden_flag 分组，marker 正式组再按 id/3000 分页

不同域的切页策略不同，但都围绕同一个目标：**让客户端能用最少的请求拿到最细粒度
的增量**。

### 3.1 marker（点位）——最复杂

marker 是数据量最大的域，切页策略在 `marker_doc.rs`：

1. 查所有正式点位（`find_safety` 已过滤软删除）；
1. **按 `hidden_flag` 分组**（`BTreeMap`，升序）——可见性隔离是第一优先级，详见
   [标记文档](./hidden-and-special-flags.md)。普通玩家只下 `Visible` 组，测试
   服玩家才下其他组；
1. **正式组（`hidden_flag = Visible = 0`）再按 `id / 3000` 切页**——每个点位 id 落在
   `[0,3000)`、`[3000,6000)`、`[6000,9000)` … 哪个区间，就属于哪一页；
1. **其他 flag 组不切页**，整组一个 MD5（这些组数据量小，没必要切）；
1. 每页各自走「序列化 → GZIP → MD5」。

为什么正式组要按 `id / 3000` 切、而不是按个数等分？因为 id 是**稳定**的：一个点位
拿到 id 后永远属于同一页，新增点位只会在末尾产生新页或填入当前末页，**已存在的页
的 MD5 不会因为别处新增而漂移**。如果改成「每 3000 个一刀」的顺序等分，那么中间
插入一个点就会让后续所有页边界移动、MD5 全变，客户端的增量同步就退化成全量重下。
按 id 哈希到固定页是内容寻址缓存的标准技巧。

### 3.2 为什么是 3000

`MARKER_PAGE_SIZE = 3000`（`marker_doc.rs`）是在**页数（请求次数）**和**单页下载量**
之间取的平衡：

- 太小（比如 300）：页数太多，客户端首屏要发几十上百个请求，握手开销和尾延迟叠加；
- 太大（比如 30000）：单页 gzip blob 几 MB，任意一个点位的改动都让整页 MD5 变、
  客户端要重下一大坨，增量失效。

3000 大致对应「单页 gzip 后几百 KB、全服 marker 几页到十几页」这个甜区。这个数和
Java 侧 `refreshMarkerBinaryList` 完全一致，**改它会同时动到前端增量同步协议**，
不是能随便调的参数。

### 3.3 item（物品）——按 flag 分组，不切页

`item_doc.rs`：物品数量远少于 marker，**每个 `hidden_flag` 组就是一个页**（index 0），
组内不再切。逻辑更简单：分组 → 每组各自压缩算 MD5。

### 3.4 MarkerLinkage 与 Tag——单 blob，不分页

`marker_link_doc.rs` 是**单 blob 变体**：整个数据集一个 MD5，不按 flag 分、不切页。
它还提供两个视图：

- `all_list_bin`：扁平的关联边数组；
- `all_graph_bin`：邻接表（`marker_id → [linked_marker_id, ...]`），客户端用来渲染
  「这个神瞳和那个宝箱是一组」「这条锄地路线按顺序连起来」的连线。

Tag（图标分类）与 icon（图标归档）同样是单 blob。这几类数据量小、且整体性强
（拆开没意义），所以一个 MD5 搞定。

## 4. 每域两个端点：MD5 清单 + 字节流

每个归档域对外暴露**成对的两个端点**，构成客户端增量同步协议：

| 端点 | 返回 | 用途 |
| --- | --- | --- |
| `list_page_bin_md5` | `[{ md5, time }]` 数组 | 客户端比对本地缓存，找出哪些页变了 |
| `list_page_bin/{md5}` | `application/octet-stream` 原始 gzip 字节 | 只下载变更页 |

`BinaryMd5Vo`（`binary_doc.rs`）的 `time` 字段是这一批页的生成时间戳（毫秒），
供客户端排序 / 展示「数据更新于」。客户端的同步流程：

```text
1. GET /marker_doc/list_page_bin_md5        →  [{md5:"a1b2..",time}, {md5:"c3d4..",time}, ...]
2. 比对本地 IndexedDB / 磁盘里缓存的 MD5 集合
3. 对每个本地没有 / 不一致的 md5：
       GET /marker_doc/list_page_bin/a1b2..  →  gzip 字节流
4. 解压、合并进本地数据集，更新 MD5 索引
```

marker 因为分了 flag 组 + id 页，清单里有多个 MD5；item 按 flag 组，每组一个；
`MarkerLinkage` / icon / Tag 等单 blob 域清单里只有一个 MD5（端点用 `all_bin` /
`all_list_bin` 等命名而非 `list_page_bin`，以示区别）。

## 5. Java vs Rust：缓存层的实现差异

Java 侧用 **Caffeine 的 `neverRefreshCacheManager`**：一旦某页的压缩字节算出来，
就以 MD5 为键缓存进 Caffeine，**永不主动失效**（数据变更靠显式调 cache 刷新端点
驱逐）。这样 `list_page_bin/{md5}` 命中的是内存里现成的字节流，零计算开销。

Rust 侧已在 `binary_doc.rs` 落地**双层缓存**，语义与 Java 对齐并扩展了跨副本能力：

- **进程内 moka 缓存**：页级 `BIN_CACHE`（容量 10000）缓存单页压缩字节；结果级
  `RESULT_CACHE`（容量 64）缓存整个域的页集合，保证 `list_page_bin_md5` 热命中时
  **零数据库查询**，不必重新全表扫描；
- **Redis 二级缓存**：域页集合序列化后写入 Redis（TTL 与进程内缓存一致，均为
  3600s），多副本部署时任一副本算过、其他副本直接命中；
- **epoch 版本化失效**：数据写入后由 `cache.rs` 的域刷新端点触发（见
  [架构概览](../guides/architecture.md#redis-与-minio-集成点)）：刷新时对
  `binmd5:epoch` 执行 INCR，所有副本据此丢弃陈旧拷贝，无需逐键扫描删除。

与 Java 的差异主要有两点：Rust 缓存带 TTL（1 小时）兜底，防止刷新端点漏调时数据
无限期陈旧；Redis 二级缓存让多副本共享计算结果，而 Caffeine 是纯单实例的进程内
缓存。

## 6. 与 hidden_flag 的耦合（重要）

切页策略把 `hidden_flag` 当成第一级分组键，意味着**可见性过滤下沉到了归档层**：
普通玩家客户端根本不会看到 `Hidden`/`Beta`/`Suprise` 组的 MD5（前端按
`userDataLevel` 请求头只拉自己有权的那几组）。这是防剧透和测试服隔离的物理隔离——
不只是查询时过滤，而是连数据分片都不下发。详见
[隐藏/特殊标记文档](./hidden-and-special-flags.md#2-hidden_flag数据级可见性过滤)。

## 7. 与 Java 实现的对齐

| Java（`genshin-map-cloud`） | Rust |
| --- | --- |
| `CompressUtils` + `DigestUtils.md5` | `binary_doc::serialize_compress_md5` |
| `MarkerDaoImpl.refreshMarkerBinaryList` | `marker_doc::do_list_page_bin_md5` / `do_list_page_bin` |
| `ItemDaoImpl.refreshItemBinaryList` | `item_doc::do_list_page_bin_md5` / `do_list_page_bin` |
| `MarkerLinkageDocController` | `marker_link_doc::do_all_list_bin_md5` 等 |
| Caffeine `neverRefreshCacheManager` | moka 进程内缓存 + Redis 二级缓存（TTL 3600s，epoch 版本化失效，见[第 5 节](#5-java-vs-rust缓存层的实现差异)） |
| `BinaryMD5Vo { md5, time }` | `BinaryMd5Vo { md5, time }`（`serde camelCase`） |

压缩字节、MD5 数值、切页边界（`id / 3000`）、flag 分组顺序都与 Java 一致，因此
Rust 后端可以和现有 Java 前端无缝对接，前端无需感知后端语言切换。

## 8. 已知简化与后续待办

- **冷启动的计算成本**：moka 与 Redis 双层缓存保证了热命中零数据库查询，但缓存
  全冷（或 TTL 过期后的首个请求）时仍要全表扫描 + 全量压缩。可考虑把
  「MD5 → 压缩字节」的映射在数据写入时就预算好落库，避免读时计算。
- **其他域的缓存刷新端点**：area / item_common / icon_tag / notice 尚无进程内
  缓存层，`cache.rs` 中对应刷新端点目前是 no-op，待这些域引入缓存后接线。
