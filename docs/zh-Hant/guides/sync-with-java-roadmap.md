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
| 3 | **notice / history（route 除外）** | `notice`（validity 排序規則）、`history`。Java 的 `route` 域**有意不移植**：`route` 公共模型留在 `models/common/` 以保持 schema 對齊，但 `/api/route` 端點不提供 | 低中 — 結構相對獨立 | **已完成** |
| 4 | **打點審批流 + 評分** | `punctuate`、`punctuate_audit`（pass/reject/delete，含角色校驗與事務化晉升）、`score`（data/generate，字段級加權） | 高 — 狀態機 + 生成邏輯 | **打點審批流已棄用**（暫存表 `marker_punctuate` 隨 schema 保留）；`score` 已完成 |
| 5 | **系統域** | `user`、`role`、`device`、`invitation`、`action_log`、`archive`（rename/delete_slot 已補齊） | 中 — 鑑權與權限耦合 | **已完成** — 登錄設備登記 + access_policy 校驗已接線 |
| 6 | **BinaryMD5 歸檔導出** | `item_doc`、`marker_doc`、`marker_link_doc` 的 bin/md5 端點；GZIP 壓縮 + 雙層緩存（moka 進程內 + Redis 二級，TTL 3600s） | 高 — 二進制協議還原 | **已完成** |
| 7 | **OAuth2 / JWKS** | `oauth` 路由（password / QQ / client_credentials）、`/.well-known/jwks.json`、access_policy 檢查、scope 映射 | 高 — 安全敏感 | **已完成** — RS256 簽名與 RSA JWKS；**JWK 輪換**經 `JWT_RSA_VERIFY_KEYS` 實現（歷史公鑰保持可驗證並隨 JWKS 發佈；由 `tests/rust/tests/jwks_rotation_test.rs` 覆蓋）；HS256 模式下 JWKS 返回空 key set（不泄露 HMAC 密鑰） |

## 當前狀態

- 全部七個批次主體已落地，五層貫通（實體 → DTO → 業務 → 路由 → 測試）。

業務斷言由 `tests/rust/tests/api_db_test.rs`（真庫，CI `integration` job）覆蓋：
area 增刪查、`item_doc` BinaryMD5、marker tweak、OAuth 策略/設備/QQ 登錄、JWKS、
緩存穩定與刷新。（打點審批流已棄用，`punctuate`/`punctuate_audit` 路由與模型已移除。）

- `SafeEntityTrait` + `impl_safe_operation!` 宏穩定，新域套用模板即可。

## 已知差距（與 Java 的剩餘差異）

- HS256 模式下 JWKS 為空 key set（簽名密鑰不對外公佈）；JWK 輪換僅 RS256 模式提供（見批次 7，覆蓋於 `tests/rust/tests/jwks_rotation_test.rs`）。
- 資料庫 schema 與真實庫的偏差待資料驗證（`marker_linkage` 空值列、

`sys_user_archive` 結構綁定等）。

- 文件翻譯：`docs/` 僅保留 en / zh-Hans / zh-Hant 三種語言並同步維護。

## 2026-10 對齊波次（Java 凍結倉 + map_register_v3 最新前端）

對照 Java `dev-single`（2026-09 凍結）與 `map_register_v3` `origin/dev`
（2026-09-23，alova + 靜態 protobuf 解碼器）逐端點復審後補齊的差異（詳見
zh-Hans 版逐條記錄）：

- `GET /api/marker_doc/list_markers`：全量點位導出（GZIP 的

`MarkerVoList` protobuf + creator/updater 使用者小表）。

- `GET /api/res/get` 與 `PUT /api/res/upload/image`：去重上傳契約

（物件鍵 = 客戶端 filePath，重複上傳顯式報錯）。

- `DELETE /api/cache/icon`：補齊 Java `CacheController` 契約；

`iconTag` 舊通道保留。

- icon 域 VO `name` → `tag` 並補齊全欄位。
- `POST /api/marker/get/list_byid` 接受雙形態請求體。
- 信封補齊：`users` 全欄位、item/icon 列表接入、分頁補 `size`。
- oauth token 補 GET 方法與 urlencoded body 通道。
- 邀請碼查詢返回精簡 VO；`HistoryOperationType` 補 IconTag=5；

marker 新增請求欄位放寬為 serde 缺省。

## 2026-10 第二波（遺留項落實）

- **history 寫路徑補齊**：marker 更新/刪除/tweak 與 item 更新/刪除現在

落 history 行（記錄**變更前**狀態的 camelCase DTO JSON，含 md5）。

- **score 生成對齊 Java 演算法**：相鄰歷史對差分（fields 按欄位計次、

chars 按 `markerTitle`/content 的 Levenshtein 距離）、首尾比對條目合成、
記帳歸屬「前一條建立者 × Asia/Shanghai 日桶」。

- **圖片直鏈統一改寫**：`ICON_PROXY_BASE` 現對 icon 響應、res 的

`fileUrl` 與 `tag_doc` 一併生效。

- **`item_type` 更新請求** hiddenFlag 放寬為 serde 缺省。
- **WS 契約更新**：確認最新前端已遷 socket.io；遷移前本地樹的

`?token=` 補丁在前端倉 `fix/ws-token-query` 分支。

- **WS 遷移決策落地**：後端經 `socketioxide` 提供 Socket.IO 兼容通道
  （`/socket.io`），契約與鑑權規則見 ws-frontend-contract 決策。

## 跟進事項

- 上述差距項排入迭代 backlog（見根目錄 `PLAN.md`），

隨 master-based PR 流程逐項合入。
