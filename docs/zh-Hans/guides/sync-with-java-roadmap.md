# Java 同步路线图

> [← 返回索引](../README.md) · 单域移植步骤见 [域同步模板](./domain-sync-template.md)

Rust 后端的目标是与 Java 参考实现 `genshin-map-cloud` 功能对齐。Java 侧的
大致范围：约 30 个控制器、约 20 个实体，覆盖地图内容域（area/icon/item/marker
及其 type/tag 变体、notice/route/history）与系统域（user/role/device/invitation/
`action_log`），以及 OAuth2/JWKS 鉴权、BinaryMD5 压缩归档导出、打点（punctuate）
审批流、评分（score）生成等能力。

移植按七个优先级批次推进，每批尽量做到可独立合并、可独立冒烟测试。

## 移植优先级

| 批次 | 域 / 特性 | 关键实体与能力 | 复杂度 | 状态 |
| --- | --- | --- | --- | --- |
| 1 | **area + marker** | `area`、`marker` 实体；CRUD + 软删除 + 乐观锁；`SafeEntityTrait` 宏定型 | 中 | **已完成** — 作为参考样板 |
| 2 | **icon / item / tag 系列** | `icon`、`icon_type`、`item`、`item_type`、`item_common`、`tag`、`tag_type`；含 copy/join/move_type、`specialFlag` 过滤 | 中高 — 实体多、关联复杂 | **已完成**（`item_doc` 等由 api_db 测试覆盖） |
| 3 | **notice / history（route 除外）** | `notice`（validity 排序规则）、`history`。Java 的 `route` 域**有意不移植**：`route` 公共模型留在 `models/common/` 以保持 schema 对齐，但 `/api/route` 端点不提供 | 低中 — 结构相对独立 | **已完成** |
| 4 | **打点审批流 + 评分** | `punctuate`、`punctuate_audit`（pass/reject/delete，含角色校验与事务化晋升）、`score`（data/generate，字段级加权） | 高 — 状态机 + 生成逻辑 | **打点审批流已弃用**（暂存表 `marker_punctuate` 随 schema 保留）；`score` 已完成 |
| 5 | **系统域** | `user`、`role`、`device`、`invitation`、`action_log`、`archive`（rename/delete_slot 已补齐） | 中 — 鉴权与权限耦合 | **已完成** — 登录设备登记 + access_policy 校验已接线 |
| 6 | **BinaryMD5 归档导出** | `item_doc`、`marker_doc`、`marker_link_doc` 的 bin/md5 端点；GZIP 压缩 + 双层缓存（moka 进程内 + Redis 二级，TTL 3600s） | 高 — 二进制协议还原 | **已完成** |
| 7 | **OAuth2 / JWKS** | `oauth` 路由（password / QQ / client_credentials）、`/.well-known/jwks.json`、access_policy 检查、scope 映射 | 高 — 安全敏感 | **已完成** — RS256 签名与 RSA JWKS；**JWK 轮换**经 `JWT_RSA_VERIFY_KEYS` 实现（历史公钥保持可验证并随 JWKS 发布；由 `tests/rust/tests/jwks_rotation_test.rs` 覆盖）；HS256 模式下 JWKS 返回空 key set（不泄露 HMAC 密钥） |

## 当前状态

- 全部七个批次主体已落地，五层贯通（实体 → DTO → 业务 → 路由 → 测试）。

业务断言由 `tests/rust/tests/api_db_test.rs`（真库，CI `integration` job）覆盖：
area 增删查、`item_doc` BinaryMD5、marker tweak、OAuth 策略/设备/QQ 登录、JWKS、
缓存稳定与刷新。（打点审批流已弃用，`punctuate`/`punctuate_audit` 路由与模型已移除。）

- `SafeEntityTrait` + `impl_safe_operation!` 宏稳定，新域套用模板即可。

## 已知差距（与 Java 的剩余差异）

- HS256 模式下 JWKS 为空 key set（签名密钥不对外公布）；JWK 轮换仅 RS256 模式提供（见批次 7，覆盖于 `tests/rust/tests/jwks_rotation_test.rs`）。
- 数据库 schema 与真实库的偏差待数据验证（`marker_linkage` 空值列、

`sys_user_archive` 结构绑定等）。

- 文档翻译：`docs/` 仅保留 en / zh-Hans / zh-Hant 三种语言并同步维护。

## 2026-10 对齐波次（Java 冻结仓 + map_register_v3 最新前端）

对照 Java `dev-single`（2026-09 冻结）与 `map_register_v3` `origin/dev`
（2026-09-23，alova + 静态 protobuf 解码器）逐端点复审后补齐的差异：

- **`GET /api/marker_doc/list_markers`**：全量点位导出（GZIP 的

`MarkerVoList` protobuf，含 creator/updater 用户小表，软删用户包含在
内 —— Java `selectUserWithDelete` 契约）。前端 4.19+ 以静态 protobuf
解码器消费；与 `list_diff_snapshot` 共用两级缓存与失效通道。

- **`GET /api/res/get`**：由 no-op 桩改为真实实现（Java

`ResourceService.getResource`）——`?filePath=` 探测 MinIO 对象，
命中返回 `{filePath, fileUrl}`，未命中返回空对象；这是前端
`getOrUploadImage` 去重上传的第一跳。

- **`PUT /api/res/upload/image`**：对象键改为**客户端 filePath**（Java

契约：同路径复用、重复上传报「当前文件已存在，请更换文件路径」），
附带路径安全校验（拒绝 `..` 穿越 / 绝对路径 / 非 `[A-Za-z0-9._-/]`
字符）。旧实现的 `uploads/{uuid}.{ext}` 随机键使去重语义不可能成立。

- **`DELETE /api/cache/icon`**：补齐 Java `CacheController` 契约

（body `List<Long>` 图标 ID 列表，空列表 = 全量清除）。前端 4.6.24 起
`cleanIconCache(number[])` 已从 `iconTag` 迁移至此；`/iconTag` 与
`/icon_tag` 旧通道保留兼容。

- **icon 域 VO 对齐**：`name` → `tag`（Java `IconVo.tag`，旧前端误用

`name`），并补齐 `creatorId/createTime/updaterId/updateTime/typeIdList/`
`urlVariants/description` 全字段；搜索请求同步接受 `tag`（alias 兼容
`name`）。

- **`POST /api/marker/get/list_byid`**：接受双形态请求体 —— 裸 ID 数组

（旧前端）与 `{"markerIdList":[…]}` 对象（Java `@RequestBody
`MarkerSearchVo``，openapi 生成的新前端）。

- **信封补齐**：`users` 映射补 qq/phone/logo/remark 全字段（去掉了多余

的 id）；item / icon 列表接入 `users`（创建者昵称列）；全部分页域补
`PageListVo.size` 字段。

- **`POST|GET /oauth/token`**：补齐 GET 方法与

`application/x-www-form-urlencoded` body 通道（Java Spring OAuth2 的
GET/POST × query/form 全兼容）；multipart（FormData 登录）与 query
（refresh）通道维持不变。

- **`/system/invitation/info`**：返回 `{code, username}` 精简 VO（Java

`SysUserInvitationSmallVo`），不再把原始实体（含 `access_policy`、
`del_flag`）序列化给公开调用方。

- **`HistoryOperationType`** 补 `IconTag=5`（Java `HistoryEnum.ICONTAG`）。
- **`POST /api/marker/single`**（新增）请求字段放宽为 serde 缺省

（`markerTitle/itemList/markerCreatorId/hiddenFlag/position`），对齐
Java 对 null 的容忍，消除新前端可选字段的 422 风险。

## 2026-10 第二波（遗留项落实）

- **history 写路径补齐**：marker 更新/删除/tweak 与 item 更新/删除现在

落 history 行（记录**变更前**状态的 camelCase DTO JSON，含 md5）；
此前 Rust 只读不写，history 列表与 score 生成均无数据源。

- **score 生成对齐 Java 算法**：相邻历史对差分（fields 按字段计次、

chars 按 `markerTitle`/content 的 Levenshtein 距离）、首尾比对条目合成
（新建点位取首个关联物品、末值取区间后首条历史或点位现状）、记账归属
「前一条创建者 × Asia/Shanghai 日桶」。

- **图片直链统一改写**：`ICON_PROXY_BASE` 现对 icon 响应、res 的

`fileUrl` 与 `tag_doc` 一并生效（ddns.minemc.top 无 CORS 头）。

- **`item_type` 更新请求** hiddenFlag 放宽为 serde 缺省（Java null 容忍）。
- **WS 契约更新**：确认最新前端已迁 socket.io（见 ws-frontend-contract

的 2026-10-22 更新）；迁移前本地树的 `?token=` 补丁在前端仓
`fix/ws-token-query` 分支。

## 跟进事项

- 上述差距项排入迭代 backlog（见根目录 `PLAN.md`），

随 master-based PR 流程逐项合入。
