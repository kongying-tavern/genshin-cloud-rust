# Java 同步路线图

> [← 返回索引](../README.md) · 单域移植步骤见 [域同步模板](./domain-sync-template.md)

Rust 后端的目标是与 Java 参考实现 `java-genshin-map-cloud` 功能对齐。Java 侧的
大致范围：约 30 个控制器、约 20 个实体，覆盖地图内容域（area/icon/item/marker
及其 type/tag 变体、notice/route/history）与系统域（user/role/device/invitation/
`action_log`），以及 OAuth2/JWKS 鉴权、BinaryMD5 压缩归档导出、打点（punctuate）
审批流、评分（score）生成等能力。

移植按七个优先级批次推进，每批尽量做到可独立合并、可独立冒烟测试。

## 移植优先级

| 批次 | 域 / 特性 | 关键实体与能力 | 复杂度 |
| --- | --- | --- | --- |
| 1 | **area + marker** | `area`、`marker` 实体；CRUD + 软删除 + 乐观锁；`SafeEntityTrait` 宏定型 | 中 — **已完成，作为参考样板** |
| 2 | **icon / item / tag 系列** | `icon`、`icon_type`、`item`、`item_type`、`item_common`；含 copy/join/move_type | 中高 — 实体多、关联复杂 |
| 3 | **notice / route / history** | `notice`、`route`、`history`（公共模型在 `models/common/`） | 低中 — 结构相对独立 |
| 4 | **打点审批流 + 评分** | `punctuate`、`punctuate_audit`（audit/delete/get）、`score`（data/generate） | 高 — 状态机 + 生成逻辑 |
| 5 | **系统域** | `user`、`role`、`device`、`invitation`、`action_log`、`archive` | 中 — 鉴权与权限耦合 |
| 6 | **BinaryMD5 归档导出** | `item_doc`、`marker_doc`、`marker_link_doc` 的 bin/md5 分页端点；GZIP 压缩 | 高 — 二进制协议还原 |
| 7 | **OAuth2 / JWKS** | `oauth` 路由 + JWKS 公钥分发 + 第三方登录 | 高 — 安全敏感 |

## 当前状态

- 批次 1 已落地：`area`、`marker` 两个域完整贯通五层（实体 → DTO → 业务 →

路由 → 冒烟测试，见 `tests/rust/tests/area/` 与 `tests/rust/tests/marker/`），
作为后续所有域的移植样板。

- `SafeEntityTrait` + `impl_safe_operation!` 宏已稳定，新域只需套用模板。

## 跟进事项

- `sea-orm` 1.x → 2.x 迁移：2.0 的 `UpdateOne`/`ValidatedUpdateOne` 破坏性

API 要求重写 `SafeEntityTrait` 宏及约 33 处业务调用点，排期为后续在 dev 分支
推进。

- `minio` 0.3 → 0.4：Client/Builder 与 bucket 置备 API 变更，待批次 6 一并处理。
