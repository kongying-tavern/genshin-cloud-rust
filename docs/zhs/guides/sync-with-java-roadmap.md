# Java 同步路线图

> [← 返回索引](../README.md) · 单域移植步骤见 [域同步模板](./domain-sync-template.md)

Rust 后端的目标是与 Java 参考实现 `java-genshin-map-cloud` 功能对齐。Java 侧的
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
| 3 | **notice / route / history** | `notice`、`route`、`history`（公共模型在 `models/common/`）；`RouteVO` 分页/搜索/批量查询 | 低中 — 结构相对独立 | **已完成** |
| 4 | **打点审批流 + 评分** | `punctuate`、`punctuate_audit`（pass/reject/delete，含角色校验与事务化晋升）、`score`（data/generate） | 高 — 状态机 + 生成逻辑 | **大部分完成** — score 的字段级 diff（`ScoreDataPunctuateVo`）尚未移植（当前为简化聚合） |
| 5 | **系统域** | `user`、`role`、`device`、`invitation`、`action_log`、`archive`（rename/delete_slot 已补齐） | 中 — 鉴权与权限耦合 | **已完成** — 登录设备登记 + access_policy 校验已接线 |
| 6 | **BinaryMD5 归档导出** | `item_doc`、`marker_doc`、`marker_link_doc` 的 bin/md5 端点；GZIP 压缩 + 进程内缓存（moka，300s TTL） | 高 — 二进制协议还原 | **已完成** |
| 7 | **OAuth2 / JWKS** | `oauth` 路由（password / QQ / client_credentials）、`/.well-known/jwks.json`、access_policy 检查、scope 映射 | 高 — 安全敏感 | **大部分完成** — 仍为 HMAC(HS256) 签名；RSA 密钥对与 JWK 轮换尚未实现 |

## 当前状态

- 全部七个批次已落地，五层贯通（实体 → DTO → 业务 → 路由 → 测试）。
  业务断言由 `tests/rust/tests/api_db_test.rs`（真库，CI `integration` job）覆盖：
  area 增删查、item_doc BinaryMD5、marker tweak、punctuate 审核（角色闸门 +
  事务晋升）、OAuth 策略/设备/QQ 登录、JWKS、缓存稳定与刷新。
- `SafeEntityTrait` + `impl_safe_operation!` 宏稳定，新域套用模板即可。

## 已知差距（与 Java 的剩余差异）

- 批次 4：`score` 的 `do_generate_score` 为简化聚合（按编辑次数计分），
  Java 的字段级 diff（`ScoreDataPunctuateVo`）未移植。
- 批次 7：token 签名为 HMAC-SHA256；Java 侧为 RSA 密钥对 + JWK 轮换。
  当前 JWKS 以 `oct` 形式公布 HMAC 密钥；切换 RSA 时需引入密钥管理与轮换。
- 数据库 schema 与真实库的偏差待数据验证（`marker_linkage` 空值列、
  `sys_user_archive` 结构绑定等）。
- 文档翻译：`docs/` 下仅 en/zhs 完整，其余 9 种语言为骨架。

## 跟进事项

- 批次 4 / 7 的差距项排入迭代 backlog（见根目录 `PLAN.md`），
  随 master-based PR 流程逐项合入。
