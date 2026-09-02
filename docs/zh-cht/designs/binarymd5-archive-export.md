<!-- markdownlint-disable MD033 MD041 -->

# BinaryMD5 歸檔導出管線

> [← 設計文檔索引](./README.md) · 相關：[隱藏/特殊標記](./hidden-and-special-flags.md)

本文解釋 Rust 後端的 `*_doc` 端點家族——爲什麼客戶端冷啓動要下載一大坨壓縮
字節流、爲什麼要按 MD5 尋址、爲什麼 marker 的正式組要按 `id / 3000` 切頁，以及
這套管線在 Rust 側當前的對齊狀態。源碼位置：

- 公共壓縮工具：`packages/functions/src/functions/api/binary_doc.rs`
- marker 歸檔：`packages/functions/src/functions/api/marker_doc.rs`
- 物品歸檔：`packages/functions/src/functions/api/item_doc.rs`
- 點位關聯歸檔：`packages/functions/src/functions/api/marker_link_doc.rs`
- 圖標 / 標籤歸檔：`packages/functions/src/functions/api/icon_doc.rs` / `tag_doc.rs`

## 1. 爲什麼存在：冷啓動的全量拉取問題

打開空熒酒館地圖客戶端的瞬間，前端要拿到**全服所有點位、所有物品、所有點位關聯**
才能渲染地圖。這不是幾十條數據——一個成熟版本的原神地圖，光 marker（神瞳、寶箱、
奇饋寶箱、採集物、地籠、解謎機關……）就有數千上萬個 POI，遍佈蒙德、璃月、稻妻、
須彌、楓丹、納塔、至冬等所有地區。逐條 JSON-over-HTTP 拉取（哪怕每頁 100 條）
意味著幾十上百個串行請求，首屏會卡到不可用。

所以後端提供的是 **GZIP 壓縮的整體 JSON blob**：把一整組實體序列化成 JSON、壓成
gzip、以壓縮後字節的 MD5 作爲鍵來尋址。客戶端只需先拉一份「MD5 清單」（很小的
JSON 數組），比對本地已緩存的 MD5，**只下載發生變化的頁**。這在三個層面受益：

- **壓縮**：點位數據高度重複（座標格式、物品 ID、圖標 ID 都是小整數或短串），gzip
  後體積通常能壓到原 JSON 的 10%~20%，移動端流量友好；
- **增量**：MD5 內容尋址讓客戶端天然支持「只取變更頁」——絕大多數冷啓動其實是
  熱啓動，本地 MD5 命中後一個字節都不用下；
- **冪等**：相同數據算出的 MD5 永遠一致，CDN / 對象存儲可直接按 MD5 鍵長緩存。

這對應 Java 側的 `CompressUtils` + `DigestUtils` + Caffeine 緩存管線，是社區地圖
工具冷啓動性能的關鍵基礎設施。

## 2. 壓縮管線：序列化 → GZIP → MD5

核心三步在 `binary_doc.rs::serialize_compress_md5` 裏，和 Java `CompressUtils`
嚴格對齊：

```rust
// 1. 序列化爲 JSON（UTF-8 字節）
let json = serde_json::to_vec(data)?;
// 2. GZIP 壓縮
let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
encoder.write_all(&json)?;
let compressed = encoder.finish()?;
// 3. 對【壓縮後字節】算 MD5（小寫 hex，32 字符）
let digest = md5::compute(&compressed);
let md5_hex = format!("{:x}", digest);
```

一個常被忽視但很關鍵的點：**MD5 是對壓縮後字節算的，不是對原始 JSON 算的**。
這意味著 MD5 同時是「內容指紋」和「壓縮產物」的鍵——客戶端拿到 `list_page_bin/{md5}`
返回的字節流，可以直接 GZIP 解壓得到原始 JSON，無需額外元數據。

## 3. 切頁策略：按 hidden_flag 分組，marker 正式組再按 id/3000 分頁

不同域的切頁策略不同，但都圍繞同一個目標：**讓客戶端能用最少的請求拿到最細粒度
的增量**。

### 3.1 marker（點位）——最複雜

marker 是數據量最大的域，切頁策略在 `marker_doc.rs`：

1. 查所有正式點位（`find_safety` 已過濾軟刪除）；
1. **按 `hidden_flag` 分組**（`BTreeMap`，升序）——可見性隔離是第一優先級，詳見
   [標記文檔](./hidden-and-special-flags.md)。普通玩家只下 `Visible` 組，測試
   服玩家才下其他組；
1. **正式組（`hidden_flag = Visible = 0`）再按 `id / 3000` 切頁**——每個點位 id 落在
   `[0,3000)`、`[3000,6000)`、`[6000,9000)` … 哪個區間，就屬於哪一頁；
1. **其他 flag 組不切頁**，整組一個 MD5（這些組數據量小，沒必要切）；
1. 每頁各自走「序列化 → GZIP → MD5」。

爲什麼正式組要按 `id / 3000` 切、而不是按個數等分？因爲 id 是**穩定**的：一個點位
拿到 id 後永遠屬於同一頁，新增點位只會在末尾產生新頁或填入當前末頁，**已存在的頁
的 MD5 不會因爲別處新增而漂移**。如果改成「每 3000 個一刀」的順序等分，那麼中間
插入一個點就會讓後續所有頁邊界移動、MD5 全變，客戶端的增量同步就退化成全量重下。
按 id 哈希到固定頁是內容尋址緩存的標準技巧。

### 3.2 爲什麼是 3000

`MARKER_PAGE_SIZE = 3000`（`marker_doc.rs`）是在**頁數（請求次數）**和**單頁下載量**
之間取的平衡：

- 太小（比如 300）：頁數太多，客戶端首屏要發幾十上百個請求，握手開銷和尾延遲疊加；
- 太大（比如 30000）：單頁 gzip blob 幾 MB，任意一個點位的改動都讓整頁 MD5 變、
  客戶端要重下一大坨，增量失效。

3000 大致對應「單頁 gzip 後幾百 KB、全服 marker 幾頁到十幾頁」這個甜區。這個數和
Java 側 `refreshMarkerBinaryList` 完全一致，**改它會同時動到前端增量同步協議**，
不是能隨便調的參數。

### 3.3 item（物品）——按 flag 分組，不切頁

`item_doc.rs`：物品數量遠少於 marker，**每個 `hidden_flag` 組就是一個頁**（index 0），
組內不再切。邏輯更簡單：分組 → 每組各自壓縮算 MD5。

### 3.4 MarkerLinkage 與 Tag——單 blob，不分頁

`marker_link_doc.rs` 是**單 blob 變體**：整個數據集一個 MD5，不按 flag 分、不切頁。
它還提供兩個視圖：

- `all_list_bin`：扁平的關聯邊數組；
- `all_graph_bin`：鄰接表（`marker_id → [linked_marker_id, ...]`），客戶端用來渲染
  「這個神瞳和那個寶箱是一組」「這條鋤地路線按順序連起來」的連線。

Tag（圖標分類）與 icon（圖標歸檔）同樣是單 blob。這幾類數據量小、且整體性強
（拆開沒意義），所以一個 MD5 搞定。

## 4. 每域兩個端點：MD5 清單 + 字節流

每個歸檔域對外暴露**成對的兩個端點**，構成客戶端增量同步協議：

| 端點 | 返回 | 用途 |
| --- | --- | --- |
| `list_page_bin_md5` | `[{ md5, time }]` 數組 | 客戶端比對本地緩存，找出哪些頁變了 |
| `list_page_bin/{md5}` | `application/octet-stream` 原始 gzip 字節 | 只下載變更頁 |

`BinaryMd5Vo`（`binary_doc.rs`）的 `time` 字段是這一批頁的生成時間戳（毫秒），
供客戶端排序 / 展示「數據更新於」。客戶端的同步流程：

```text
1. GET /marker_doc/list_page_bin_md5        →  [{md5:"a1b2..",time}, {md5:"c3d4..",time}, ...]
2. 比對本地 IndexedDB / 磁盤裏緩存的 MD5 集合
3. 對每個本地沒有 / 不一致的 md5：
       GET /marker_doc/list_page_bin/a1b2..  →  gzip 字節流
4. 解壓、合併進本地數據集，更新 MD5 索引
```

marker 因爲分了 flag 組 + id 頁，清單裏有多個 MD5；item 按 flag 組，每組一個；
`MarkerLinkage` / icon / Tag 等單 blob 域清單裏只有一個 MD5（端點用 `all_bin` /
`all_list_bin` 等命名而非 `list_page_bin`，以示區別）。

## 5. Java vs Rust：緩存層的實現差異

Java 側用 **Caffeine 的 `neverRefreshCacheManager`**：一旦某頁的壓縮字節算出來，
就以 MD5 爲鍵緩存進 Caffeine，**永不主動失效**（數據變更靠顯式調 cache 刷新端點
驅逐）。這樣 `list_page_bin/{md5}` 命中的是內存裏現成的字節流，零計算開銷。

Rust 側已在 `binary_doc.rs` 落地**雙層緩存**，語義與 Java 對齊並擴展了跨副本能力：

- **進程內 moka 緩存**：頁級 `BIN_CACHE`（容量 10000）緩存單頁壓縮字節；結果級
  `RESULT_CACHE`（容量 64）緩存整個域的頁集合，保證 `list_page_bin_md5` 熱命中時
  **零數據庫查詢**，不必重新全表掃描；
- **Redis 二級緩存**：域頁集合序列化後寫入 Redis（TTL 與進程內緩存一致，均爲
  3600s），多副本部署時任一副本算過、其他副本直接命中；
- **epoch 版本化失效**：數據寫入後由 `cache.rs` 的域刷新端點觸發（見
  [架構概覽](../guides/architecture.md#redis-與-minio-集成點)）：刷新時對
  `binmd5:epoch` 執行 INCR，所有副本據此丟棄陳舊拷貝，無需逐鍵掃描刪除。

與 Java 的差異主要有兩點：Rust 緩存帶 TTL（1 小時）兜底，防止刷新端點漏調時數據
無限期陳舊；Redis 二級緩存讓多副本共享計算結果，而 Caffeine 是純單實例的進程內
緩存。

## 6. 與 hidden_flag 的耦合（重要）

切頁策略把 `hidden_flag` 當成第一級分組鍵，意味著**可見性過濾下沉到了歸檔層**：
普通玩家客戶端根本不會看到 `Hidden`/`Beta`/`Suprise` 組的 MD5（前端按
`userDataLevel` 請求頭只拉自己有權的那幾組）。這是防劇透和測試服隔離的物理隔離——
不只是查詢時過濾，而是連數據分片都不下發。詳見
[隱藏/特殊標記文檔](./hidden-and-special-flags.md#2-hidden_flag數據級可見性過濾)。

## 7. 與 Java 實現的對齊

| Java（`genshin-map-cloud`） | Rust |
| --- | --- |
| `CompressUtils` + `DigestUtils.md5` | `binary_doc::serialize_compress_md5` |
| `MarkerDaoImpl.refreshMarkerBinaryList` | `marker_doc::do_list_page_bin_md5` / `do_list_page_bin` |
| `ItemDaoImpl.refreshItemBinaryList` | `item_doc::do_list_page_bin_md5` / `do_list_page_bin` |
| `MarkerLinkageDocController` | `marker_link_doc::do_all_list_bin_md5` 等 |
| Caffeine `neverRefreshCacheManager` | moka 進程內緩存 + Redis 二級緩存（TTL 3600s，epoch 版本化失效，見[第 5 節](#5-java-vs-rust緩存層的實現差異)） |
| `BinaryMD5Vo { md5, time }` | `BinaryMd5Vo { md5, time }`（`serde camelCase`） |

壓縮字節、MD5 數值、切頁邊界（`id / 3000`）、flag 分組順序都與 Java 一致，因此
Rust 後端可以和現有 Java 前端無縫對接，前端無需感知後端語言切換。

## 8. 已知簡化與後續待辦

- **冷啓動的計算成本**：moka 與 Redis 雙層緩存保證了熱命中零數據庫查詢，但緩存
  全冷（或 TTL 過期後的首個請求）時仍要全表掃描 + 全量壓縮。可考慮把
  「MD5 → 壓縮字節」的映射在數據寫入時就預算好落庫，避免讀時計算。
- **其他域的緩存刷新端點**：area / item_common / icon_tag / notice 尚無進程內
  緩存層，`cache.rs` 中對應刷新端點目前是 no-op，待這些域引入緩存後接線。
