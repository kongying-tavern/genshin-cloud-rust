# 領域術語表（Glossary）

> [← 返回索引](../README.md) · 相關：[架構概覽](./architecture.md) · [設計文檔](../designs/README.md)

本表收錄「空熒酒館·原神地圖」業務語境下的高頻術語，給出**中文叫法 ↔ 英文 / 官方譯名 ↔ 代碼標識符**的對照，並附一句在地圖工具裏它具體指什麼。新人看代碼、讀設計文檔、對齊 Java 參考實現時都可參照此表。

代碼標識符一欄給出 sea-orm 實體路徑（`packages/database/src/models/...`）或枚舉路徑（`packages/utils/src/types/...`），便於直接跳轉。

## 一、原神遊戲內容（地圖標註對象）

| 中文 | 英文 / 官方譯名 | 代碼標識符 / 說明 |
| --- | --- | --- |
| 神瞳 | Oculus（風/巖/雷/草/水/火神瞳） | 玩家供奉七天神像的核心收集品。地圖上數量最多的點位類型之一，圖標走 `IconStyleType::Oculus`（類神瞳無對勾樣式） |
| 寶箱 | Chest | 普通寶箱 / 精緻寶箱 / 珍貴寶箱 / 奇饋寶箱等，作爲 `marker` 上掛的 `item` |
| 奇饋寶箱 | Remarkable Chest | 須彌特有的會掉落擺設圖紙的寶箱，地圖工具重點標註對象 |
| 地籠（地靈龕） | Shrine of Depth | 各國分佈的、需對應鑰匙開啓的華麗寶箱龕 |
| 採集物 | Material / Local Specialty | 特產（如璃月特產霓裳花）、食材、礦石等刷新型 `item`，帶 `default_refresh_time` |
| 鐵礦 / 水晶礦 / 白鐵礦 | Iron Chunk / Crystal Chunk / White Iron Chunk | 礦石類採集物，按刷新時間週期性出現 |
| 鋤地（鋤大地） | Artifact / Boss farming route | 玩家巡線刷怪的玩法，對應 `route`（路線）實體，`marker_list` 是有序點位數組 |
| 仙靈 | Seelie | 需引導回仙靈座的解謎點位，常作爲 `marker_linkage`（點位關聯）連成一組 |
| 挑戰 | Time Trial Challenge | 限時挑戰牌，完成後給寶箱 |
| 解謎 | Puzzle | 各種機關解謎，完成後常解鎖寶箱或神瞳 |
| 七天神像 | Statue of The Seven | 供奉神瞳、回覆血量的關鍵地標 |
| 傳送錨點 | Teleport Waypoint | 玩家解鎖後可快速傳送的點位 |
| 突破素材 | Ascension Material | 角色 / 武器突破用素材，按 boss / 世界 boss 分類 |
| 風魔龍 / 公子 / 若陀龍王等 | Weekly Boss | 每週限次的高收益 boss，地圖上單點標註 |
| 夢之樹 | Dream Tree / Fountain of Lucine | 須彌 / 楓丹等供奉處，供奉道具換獎勵 |
| 釣魚點 | Fishing Spot | 各國魚類分佈點，按魚種標註 |
| 提瓦特 | Teyvat | 原神遊戲世界的總稱 |

> 提示：遊戲內容會隨版本更新擴充（楓丹、納塔、至冬等），這些新地區的點位在錄入階段會標 `hidden_flag = Beta`（測試服），上線當天轉 `Visible`，詳見 [隱藏/特殊標記設計](../designs/hidden-and-special-flags.md)。

## 二、地圖工具業務概念

| 中文術語 | 英文 / 代碼標識符 | 含義 |
| --- | --- | --- |
| 點位 | `marker`（`models/marker/marker.rs`） | 地圖上一個帶座標、圖標、物品的標記點。核心實體 |
| 物品 | `item`（`models/item/item.rs`） | 可被點位掛載的收集對象，關聯圖標、地區、刷新時間 |
| 圖標 | `icon`（`models/icon/icon.rs`） | 標記的視覺資源（PNG / SVG），帶 `url`、`tag`、`url_variants`（多分辨率變體） |
| 地區 | `area`（`models/area/area.rs`） | 遊戲地區（蒙德 / 璃月 / 稻妻 / 須彌 / 楓丹 / 納塔...），樹形結構（`parent_id`、`is_final` 末端地區） |
| 路線 | `route`（`models/common/route.rs`） | 鋤地路線，`marker_list` 是有序 `marker_id` 數組，前端按順序連線 |
| 標籤 / 圖標分類 | `tag` / `tag_type`（`models/tag/`） | `tag` 把 `icon` 歸到 `tag_type` 下；`tag_type` 是樹形分類（`parent_id`、`is_final`） |
| 物品類型 | `item_type`（`models/item/item_type.rs`） | 物品的分類維度，通過 `item_type_link` 多對多關聯 |
| 點位關聯 | `marker_linkage`（`models/marker/marker_linkage.rs`） | 點位之間的關係（同組、觸發、路徑等），歸檔時導出 `list`（邊數組）與 `graph`（鄰接表）兩個視圖 |
| 點位-物品關聯 | marker-item link / `marker_item_link` | 點位與物品的多對多關係 |
| 打點（提交） | punctuate / `marker_punctuate`（`models/marker/marker_punctuate.rs`） | 玩家貢獻點位的一次提交動作，暫存表記錄（審核工作流已棄用，暫存表隨 schema 保留） |
| 貢獻評分 | `score_stat`（`models/common/score_stat.rs`） | 按 `scope` + `span` 聚合的貢獻者評分，打點通過後匯入 |
| BinaryMD5 歸檔 | `*_doc` 端點家族 | GZIP 壓縮的批量數據導出，以壓縮字節 MD5 爲鍵，供客戶端冷啓動增量同步，詳見 [歸檔導出](../designs/binarymd5-archive-export.md) |
| 歷史記錄 | `history`（`models/common/history.rs`） | 編輯操作的審計軌跡，`edit_type`（新增/修改/刪除）+ `operation_type`（地區/圖標/物品/點位） |
| 公告 | `notice`（`models/common/notice.rs`） | 站點公告，客戶端啓動時拉取 |
| 資源上傳 | `res` / MinIO | 玩家上傳的點位截圖 / 視頻，存對象存儲 MinIO |
| 空熒酒館 | Kongying Tavern | 運營本地圖工具的社區組織 |

## 三、狀態與標記字段

| 中文術語 | 代碼標識符 | 取值與含義 |
| --- | --- | --- |
| 暫存 | `MarkerPunctuateStatus::Pending`（`0`） | 打點建議草稿狀態（對應 Java STAGE），編輯側不可見 |
| 審核中 | `MarkerPunctuateStatus::Reviewing`（`1`） | 已提交（對應 Java COMMIT），等待編輯處理 |
| 不通過 | `MarkerPunctuateStatus::Rejected`（`2`） | 編輯駁回（對應 Java REJECT，附 `audit_remark`），**非終態**，可改後重提 |
| 新增 / 修改 / 刪除 | `MarkerPunctuateMethodType::{Added, Modified, Deleted}`（1/2/3） | 打點建議的操作類型，決定晉升路徑 |
| 可見 | `HiddenFlag::Visible`（`0`） | 所有人可見 |
| 隱藏 | `HiddenFlag::Hidden`（`1`） | 僅測試 / 後臺 |
| 測試服 | `HiddenFlag::Beta`（`2`，舊名 `Spy`） | 僅測試服玩家（未上線的新版本數據） |
| 彩蛋 | `HiddenFlag::Suprise`（`3`） | 主動開啓彩蛋的玩家可見（防劇透） |
| 測試服角色 | Beta（`SystemUserRole::MapBeta`，舊名 `MapNeigui`） | 擁有測試服數據訪問權限的用戶角色 |
| 特殊標記 | `special_flag`（i32 位掩碼） | 物品 / 地區的查詢過濾位，第 0 位 = 前臺是否默認顯示，詳見 [標記設計](../designs/hidden-and-special-flags.md) |
| 軟刪除 | `del_flag`（bool） | 邏輯刪除標記，`SafeEntityTrait::find_safety` 自動過濾 `del_flag=false` |
| 樂觀鎖 | `version`（i64） | 實體版本號，`update_safety` 靠 `WHERE version = old` 自增實現 |
| 用戶數據等級 | `userDataLevel`（請求頭，位掩碼） | 前端聲明可見的 hidden_flag 層級組合，後端按此分片下發歸檔 |

## 四、技術 / 架構術語

| 中文術語 | 英文 / 代碼標識符 | 含義 |
| --- | --- | --- |
| 安全實體 trait | `SafeEntityTrait`（`packages/utils/src/db_operations.rs`） | 統一提供 `find_safety`（過濾軟刪）/ `update_safety`（樂觀鎖）/ `delete_safety`（軟刪）的宏派生 |
| 業務函數 | `do_*`（`packages/functions/src/functions/api/*.rs`） | 每個域對外暴露的異步業務函數，路由層薄封裝後調用 |
| 鑑權信息 | `AuthInfo`（`packages/utils/src/jwt.rs`） | JWT 解出的用戶身份，路由層用 `ExtractAuthInfo` 中間件注入 |
| 通用響應包裝 | `CommonResponse`（`packages/utils/src/models/wrapper.rs`） | 統一的 `{ code, data/msg }` 響應殼 |
| 分頁包裝 | `Pagination`（`current` + `size`） | 列表查詢的通用分頁入參 |
| 二進制歸檔對象 | `BinaryMd5Vo { md5, time }` | 歸檔清單的單條目，`md5` 是壓縮字節指紋，`time` 是生成時間 |
| 冷啓動 | cold start | 客戶端首次加載，需下載全部點位 / 物品數據 |
| 增量同步 | incremental sync | 客戶端比對 MD5 列表，僅獲取變更的數據頁 |
| Caffeine 緩存 | （Java 側 `neverRefreshCacheManager`） | Java 用 Caffeine 做永不過期的進程內緩存；Rust 側以 moka 進程內緩存 + Redis 二級緩存對應實現 |
| Java 參考實現 | `genshin-map-cloud` | 本 Rust 項目的對照源，功能需與之對齊，見 [Java 同步路線圖](./sync-with-java-roadmap.md) |
| 四包分層 | `utils / database / functions / router` | 工作區自底向上分層，詳見 [架構概覽](./architecture.md#四包分層) |

## 五、原神地區速查

| 中文 | 英文 | 說明 |
| --- | --- | --- |
| 蒙德 | Mondstadt | 風之國，風神瞳（Anemoculus）分佈 |
| 璃月 | Liyue | 巖之國，巖神瞳（Geoculus）分佈 |
| 稻妻 | Inazuma | 雷之國，雷神瞳（Electroculus）分佈 |
| 淵下宮 | Enkanomiya | 稻妻地下區域 |
| 層巖巨淵 | The Chasm | 璃月地下礦區 |
| 須彌 | Sumeru | 草之國，草神瞳（Dendroculus）分佈 |
| 楓丹 | Fontaine | 水之國，水神瞳（Hydroculus）分佈 |
| 納塔 | Natlan | 火之國，火神瞳（Pyroculus）分佈 |
| 至冬 | Snezhnaya | 冰之國（劇情推進中逐步開放） |

> 地區在 `area` 表裏是樹形結構（`parent_id`、`is_final`），末端地區用於掛 `item`（物品必屬一個末端地區）。新版本地區錄入時標 `hidden_flag = Beta`，上線轉 `Visible`。

## 六、系統域

| 中文 | English | 代碼標識符 | 說明 |
| --- | --- | --- | --- |
| 系統用戶 | system user | `sys_user` | 註冊用戶 |
| 用戶設備 | user device | `sys_user_device` | 登錄設備追蹤（異常檢測） |
| 用戶邀請 | user invitation | `sys_user_invitation` | 邀請碼註冊機制 |
| 操作日誌 | action log | `sys_action_log` | 用戶操作審計 |
| 用戶存檔 | user archive | `sys_user_archive` | 客戶端存檔槽位 |

## 七、易混淆點提醒

- **`tag` vs `tag_type` vs `item_type`**：`tag` 是把**圖標**歸類到 `tag_type`（圖標分類）；`item_type` 是**物品**的分類，兩者獨立。別把「圖標的分類」和「物品的分類」混爲一談。
- **`marker` vs `marker_punctuate`**：`marker` 是全服可見的正式點位；`marker_punctuate` 是玩家提交的、待審核的打點建議暫存表（審核工作流已棄用，暫存表隨 schema 保留），兩者數據隔離。
- **`hidden_flag` vs `special_flag` vs `del_flag`**：三套正交標記，分別管「誰能看」、「查不查得出來」、「是否已刪」，詳見 [標記設計](../designs/hidden-and-special-flags.md#4-三套標記的正交關係)。
- **`Suprise` 拼寫**：`HiddenFlag::Suprise` 少一個 `r`，是爲對齊 Java 歷史拼寫、保證庫存數據反序列化兼容，**不是筆誤**，勿改。
- **`IconStyleType::LikeOculus`（值 2）已廢棄**：保留僅爲兼容歷史 `num_value=2` 的行，新數據應使用 `Oculus`（值 3）。
