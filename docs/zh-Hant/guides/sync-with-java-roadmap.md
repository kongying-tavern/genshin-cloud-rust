# Java 同步路線圖

> [← 返回索引](../README.md) · 單域移植步驟見 [域同步模板](./domain-sync-template.md)

Rust 後端的目標是與 Java 參考實現 `genshin-map-cloud` 功能對齊。Java 側的
大致範圍：約 30 個控制器、約 20 個實體，覆蓋地圖內容域（area/icon/item/marker
及其 type/tag 變體、notice/route/history）與系統域（user/role/device/invitation/
`action_log`），以及 OAuth2/JWKS 鑑權、BinaryMD5 壓縮歸檔導出、打點（punctuate）
審批流、評分（score）生成等能力。

移植按七個優先級批次推進，每批儘量做到可獨立合併、可獨立冒煙測試。

## 移植優先級

| 批次 | 域 / 特性 | 關鍵實體與能力 | 複雜度 | 狀態 |
| --- | --- | --- | --- | --- |
| 1 | **area + marker** | `area`、`marker` 實體；CRUD + 軟刪除 + 樂觀鎖；`SafeEntityTrait` 宏定型 | 中 | **已完成** — 作爲參考樣板 |
| 2 | **icon / item / tag 系列** | `icon`、`icon_type`、`item`、`item_type`、`item_common`、`tag`、`tag_type`；含 copy/join/move_type、`specialFlag` 過濾 | 中高 — 實體多、關聯複雜 | **已完成**（`item_doc` 等由 api_db 測試覆蓋） |
| 3 | **notice / route / history** | `notice`、`route`、`history`（公共模型在 `models/common/`）；`RouteVO` 分頁/搜索/批量查詢 | 低中 — 結構相對獨立 | **已完成** |
| 4 | **打點審批流 + 評分** | `punctuate`、`punctuate_audit`（pass/reject/delete，含角色校驗與事務化晉升）、`score`（data/generate，字段級加權） | 高 — 狀態機 + 生成邏輯 | **打點審批流已棄用**（暫存表 `marker_punctuate` 隨 schema 保留）；`score` 已完成 |
| 5 | **系統域** | `user`、`role`、`device`、`invitation`、`action_log`、`archive`（rename/delete_slot 已補齊） | 中 — 鑑權與權限耦合 | **已完成** — 登錄設備登記 + access_policy 校驗已接線 |
| 6 | **BinaryMD5 歸檔導出** | `item_doc`、`marker_doc`、`marker_link_doc` 的 bin/md5 端點；GZIP 壓縮 + 雙層緩存（moka 進程內 + Redis 二級，TTL 3600s） | 高 — 二進制協議還原 | **已完成** |
| 7 | **OAuth2 / JWKS** | `oauth` 路由（password / QQ / client_credentials）、`/.well-known/jwks.json`、access_policy 檢查、scope 映射 | 高 — 安全敏感 | **大部分完成** — RS256 簽名與 RSA JWKS 已實現（`JWT_RSA_PRIVATE_KEY_PEM`）；JWK 輪換仍未實現；HS256 模式下 JWKS 返回空 key set（不泄露 HMAC 密鑰） |

## 當前狀態

- 全部七個批次主體已落地，五層貫通（實體 → DTO → 業務 → 路由 → 測試）。
  業務斷言由 `tests/rust/tests/api_db_test.rs`（真庫，CI `integration` job）覆蓋：
  area 增刪查、item_doc BinaryMD5、marker tweak、OAuth 策略/設備/QQ 登錄、JWKS、
  緩存穩定與刷新。（打點審批流已棄用，`punctuate`/`punctuate_audit` 路由與模型已移除。）
- `SafeEntityTrait` + `impl_safe_operation!` 宏穩定，新域套用模板即可。

## 已知差距（與 Java 的剩餘差異）

- 批次 7：JWK 輪換未實現（密鑰固定，無輪換機制）；HS256 模式下 JWKS 爲空 key set（簽名密鑰不對外公佈）。
- 數據庫 schema 與真實庫的偏差待數據驗證（`marker_linkage` 空值列、
  `sys_user_archive` 結構綁定等）。
- 文檔翻譯：`docs/` 僅保留 en / zh-Hans / zh-Hant 三種語言並同步維護。

## 跟進事項

- 批次 4 / 7 的差距項排入迭代 backlog（見根目錄 `PLAN.md`），
  隨 master-based PR 流程逐項合入。
