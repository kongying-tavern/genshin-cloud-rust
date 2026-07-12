# 领域术语表（Glossary）

> [← 返回索引](../README.md) · 相关：[架构概览](./architecture.md) · [设计文档](../designs/README.md)

本表收录「空荧酒馆·原神地图」业务语境下的高频术语，给出**中文叫法 ↔ 英文 / 官方译名 ↔ 代码标识符**的对照，并附一句在地图工具里它具体指什么。新人看代码、读设计文档、对齐 Java 参考实现时都可参照此表。

代码标识符一栏给出 sea-orm 实体路径（`packages/database/src/models/...`）或枚举路径（`packages/utils/src/types/...`），便于直接跳转。

## 一、原神游戏内容（地图标注对象）

| 中文 | 英文 / 官方译名 | 代码标识符 / 说明 |
| --- | --- | --- |
| 神瞳 | Oculus（风/岩/雷/草/水/火神瞳） | 玩家供奉七天神像的核心收集品。地图上数量最多的点位类型之一，图标走 `IconStyleType::Oculus`（类神瞳无对勾样式） |
| 宝箱 | Chest | 普通宝箱 / 精致宝箱 / 珍贵宝箱 / 奇馈宝箱等，作为 `marker` 上挂的 `item` |
| 奇馈宝箱 | Remarkable Chest | 须弥特有的会掉落摆设图纸的宝箱，地图工具重点标注对象 |
| 地笼（地灵龛） | Shrine of Depth | 各国分布的、需对应钥匙开启的华丽宝箱龛 |
| 采集物 | Material / Local Specialty | 特产（如璃月特产霓裳花）、食材、矿石等刷新型 `item`，带 `default_refresh_time` |
| 铁矿 / 水晶矿 / 白铁矿 | Iron Chunk / Crystal Chunk / White Iron Chunk | 矿石类采集物，按刷新时间周期性出现 |
| 锄地（锄大地） | Artifact / Boss farming route | 玩家巡线刷怪的玩法，对应 `route`（路线）实体，`marker_list` 是有序点位数组 |
| 仙灵 | Seelie | 需引导回仙灵座的解谜点位，常作为 `marker_linkage`（点位关联）连成一组 |
| 挑战 | Time Trial Challenge | 限时挑战牌，完成后给宝箱 |
| 解谜 | Puzzle | 各种机关解谜，完成后常解锁宝箱或神瞳 |
| 七天神像 | Statue of The Seven | 供奉神瞳、回复血量的关键地标 |
| 传送锚点 | Teleport Waypoint | 玩家解锁后可快速传送的点位 |
| 突破素材 | Ascension Material | 角色 / 武器突破用素材，按 boss / 世界 boss 分类 |
| 风魔龙 / 公子 / 若陀龙王等 | Weekly Boss | 每周限次的高收益 boss，地图上单点标注 |
| 梦之树 | Dream Tree / Fountain of Lucine | 须弥 / 枫丹等供奉处，供奉道具换奖励 |
| 钓鱼点 | Fishing Spot | 各国鱼类分布点，按鱼种标注 |
| 提瓦特 | Teyvat | 原神游戏世界的总称 |

> 提示：游戏内容会随版本更新扩充（枫丹、纳塔、至冬等），这些新地区的点位在录入阶段会标 `hidden_flag = Spy`（测试服），上线当天转 `Visible`，详见 [隐藏/特殊标记设计](../designs/hidden-and-special-flags.md)。

## 二、地图工具业务概念

| 中文术语 | 英文 / 代码标识符 | 含义 |
| --- | --- | --- |
| 点位 | `marker`（`models/marker/marker.rs`） | 地图上一个带坐标、图标、物品的标记点。核心实体 |
| 物品 | `item`（`models/item/item.rs`） | 可被点位挂载的收集对象，关联图标、地区、刷新时间 |
| 图标 | `icon`（`models/icon/icon.rs`） | 标记的视觉资源（PNG / SVG），带 `url`、`tag`、`url_variants`（多分辨率变体） |
| 地区 | `area`（`models/area/area.rs`） | 游戏地区（蒙德 / 璃月 / 稻妻 / 须弥 / 枫丹 / 纳塔...），树形结构（`parent_id`、`is_final` 末端地区） |
| 路线 | `route`（`models/common/route.rs`） | 锄地路线，`marker_list` 是有序 `marker_id` 数组，前端按顺序连线 |
| 标签 / 图标分类 | `tag` / `tag_type`（`models/tag/`） | `tag` 把 `icon` 归到 `tag_type` 下；`tag_type` 是树形分类（`parent_id`、`is_final`） |
| 物品类型 | `item_type`（`models/item/item_type.rs`） | 物品的分类维度，通过 `item_type_link` 多对多关联 |
| 点位关联 | `marker_linkage`（`models/marker/marker_linkage.rs`） | 点位之间的关系（同组、触发、路径等），归档时导出 `list`（边数组）与 `graph`（邻接表）两个视图 |
| 点位-物品关联 | marker-item link / `marker_item_link` | 点位与物品的多对多关系 |
| 打点（提交） | punctuate / `marker_punctuate`（`models/marker/marker_punctuate.rs`） | 玩家贡献点位的一次提交动作，经审核后晋升为正式 `marker`，详见 [打点工作流](../designs/punctuate-workflow.md) |
| 贡献评分 | `score_stat`（`models/common/score_stat.rs`） | 按 `scope` + `span` 聚合的贡献者评分，打点通过后汇入 |
| BinaryMD5 归档 | `*_doc` 端点家族 | GZIP 压缩的批量数据导出，以压缩字节 MD5 为键，供客户端冷启动增量同步，详见 [归档导出](../designs/binarymd5-archive-export.md) |
| 历史记录 | `history`（`models/common/history.rs`） | 编辑操作的审计轨迹，`edit_type`（新增/修改/删除）+ `operation_type`（地区/图标/物品/点位） |
| 公告 | `notice`（`models/common/notice.rs`） | 站点公告，客户端启动时拉取 |
| 资源上传 | `res` / MinIO | 玩家上传的点位截图 / 视频，存对象存储 MinIO |
| 空荧酒馆 | Kongying Tavern | 运营本地图工具的社区组织 |

## 三、状态与标记字段

| 中文术语 | 代码标识符 | 取值与含义 |
| --- | --- | --- |
| 暂存 | `MarkerPunctuateStatus::Pending`（`0`） | 打点建议草稿状态（对应 Java STAGE），编辑侧不可见 |
| 审核中 | `MarkerPunctuateStatus::Reviewing`（`1`） | 已提交（对应 Java COMMIT），等待编辑处理 |
| 不通过 | `MarkerPunctuateStatus::Rejected`（`2`） | 编辑驳回（对应 Java REJECT，附 `audit_remark`），**非终态**，可改后重提 |
| 新增 / 修改 / 删除 | `MarkerPunctuateMethodType::{Added, Modified, Deleted}`（1/2/3） | 打点建议的操作类型，决定晋升路径 |
| 可见 | `HiddenFlag::Visible`（`0`） | 所有人可见 |
| 隐藏 | `HiddenFlag::Hidden`（`1`） | 仅内鬼 / 后台 |
| 测试服 | `HiddenFlag::Spy`（`2`） | 仅测试服玩家（未上线的新版本数据） |
| 彩蛋 | `HiddenFlag::Suprise`（`3`） | 主动开启彩蛋的玩家可见（防剧透） |
| 内鬼 | Insider / Spy | 拥有测试服 / 内部数据访问权限的用户角色 |
| 特殊标记 | `special_flag`（i32 位掩码） | 物品 / 地区的查询过滤位，第 0 位 = 前台是否默认显示，详见 [标记设计](../designs/hidden-and-special-flags.md) |
| 软删除 | `del_flag`（bool） | 逻辑删除标记，`SafeEntityTrait::find_safety` 自动过滤 `del_flag=false` |
| 乐观锁 | `version`（i64） | 实体版本号，`update_safety` 靠 `WHERE version = old` 自增实现 |
| 用户数据等级 | `userDataLevel`（请求头，位掩码） | 前端声明可见的 hidden_flag 层级组合，后端按此分片下发归档 |

## 四、技术 / 架构术语

| 中文术语 | 英文 / 代码标识符 | 含义 |
| --- | --- | --- |
| 安全实体 trait | `SafeEntityTrait`（`packages/utils/src/db_operations.rs`） | 统一提供 `find_safety`（过滤软删）/ `update_safety`（乐观锁）/ `delete_safety`（软删）的宏派生 |
| 业务函数 | `do_*`（`packages/functions/src/functions/api/*.rs`） | 每个域对外暴露的异步业务函数，路由层薄封装后调用 |
| 鉴权信息 | `AuthInfo`（`packages/utils/src/jwt.rs`） | JWT 解出的用户身份，路由层用 `ExtractAuthInfo` 中间件注入 |
| 通用响应包装 | `CommonResponse`（`packages/utils/src/models/wrapper.rs`） | 统一的 `{ code, data/msg }` 响应壳 |
| 分页包装 | `Pagination`（`current` + `size`） | 列表查询的通用分页入参 |
| 二进制归档对象 | `BinaryMd5Vo { md5, time }` | 归档清单的单条目，`md5` 是压缩字节指纹，`time` 是生成时间 |
| 冷启动 | cold start | 客户端首次加载，需下载全部点位 / 物品数据 |
| 增量同步 | incremental sync | 客户端比对 MD5 列表，仅获取变更的数据页 |
| Caffeine 缓存 | （Java 侧 `neverRefreshCacheManager`） | Java 用 Caffeine 做永不过期的进程内缓存；Rust 侧对应缓存层待加（候选 `moka` / Redis） |
| Java 参考实现 | `java-genshin-map-cloud` | 本 Rust 项目的对照源，功能需与之对齐，见 [Java 同步路线图](./sync-with-java-roadmap.md) |
| 四包分层 | `utils / database / functions / router` | 工作区自底向上分层，详见 [架构概览](./architecture.md#四包分层) |

## 五、原神地区速查

| 中文 | 英文 | 说明 |
| --- | --- | --- |
| 蒙德 | Mondstadt | 风之国，风神瞳（Anemoculus）分布 |
| 璃月 | Liyue | 岩之国，岩神瞳（Geoculus）分布 |
| 稻妻 | Inazuma | 雷之国，雷神瞳（Electroculus）分布 |
| 渊下宫 | Enkanomiya | 稻妻地下区域 |
| 层岩巨渊 | The Chasm | 璃月地下矿区 |
| 须弥 | Sumeru | 草之国，草神瞳（Dendroculus）分布 |
| 枫丹 | Fontaine | 水之国，水神瞳（Hydroculus）分布 |
| 纳塔 | Natlan | 火之国，火神瞳（Pyroculus）分布 |
| 至冬 | Snezhnaya | 冰之国（剧情推进中逐步开放） |

> 地区在 `area` 表里是树形结构（`parent_id`、`is_final`），末端地区用于挂 `item`（物品必属一个末端地区）。新版本地区录入时标 `hidden_flag = Spy`，上线转 `Visible`。

## 六、系统域

| 中文 | English | 代码标识符 | 说明 |
| --- | --- | --- | --- |
| 系统用户 | system user | `sys_user` | 注册用户 |
| 用户设备 | user device | `sys_user_device` | 登录设备追踪（异常检测） |
| 用户邀请 | user invitation | `sys_user_invitation` | 邀请码注册机制 |
| 操作日志 | action log | `sys_action_log` | 用户操作审计 |
| 用户存档 | user archive | `sys_user_archive` | 客户端存档槽位 |

## 七、易混淆点提醒

- **`tag` vs `tag_type` vs `item_type`**：`tag` 是把**图标**归类到 `tag_type`（图标分类）；`item_type` 是**物品**的分类，两者独立。别把「图标的分类」和「物品的分类」混为一谈。
- **`marker` vs `marker_punctuate`**：`marker` 是全服可见的正式点位；`marker_punctuate` 是玩家提交的、待审核的打点建议。晋升前两者数据隔离，详见 [打点工作流](../designs/punctuate-workflow.md#2-两个表为什么要分离)。
- **`hidden_flag` vs `special_flag` vs `del_flag`**：三套正交标记，分别管「谁能看」、「查不查得出来」、「是否已删」，详见 [标记设计](../designs/hidden-and-special-flags.md#4-三套标记的正交关系)。
- **`Suprise` 拼写**：`HiddenFlag::Suprise` 少一个 `r`，是为对齐 Java 历史拼写、保证库存数据反序列化兼容，**不是笔误**，勿改。
- **`IconStyleType::LikeOculus`（值 2）已废弃**：保留仅为兼容历史 `num_value=2` 的行，新数据应使用 `Oculus`（值 3）。
