<!-- markdownlint-disable MD033 MD041 -->

# 隱藏標記與特殊標記（Hidden / Special Flags）

> [← 設計文檔索引](./README.md) · 相關：[BinaryMD5 歸檔導出](./binarymd5-archive-export.md)

本文解釋原神地圖後端三套**正交**的可見性 / 過濾標記：`hidden_flag`（誰能看）、
`special_flag`（查不查得出來）、`del_flag`（軟刪除）。它們名字相近、都掛在點位 /
物品 / 地區實體上，但語義完全不同，混用會把測試服數據漏給正式服玩家、或把已刪點位
重新暴露給全服。源碼位置：

- 枚舉定義：`packages/utils/src/types/common.rs`（`HiddenFlag`）
- 實體字段：`marker.rs` / `item.rs` / `area.rs`（`hidden_flag` / `special_flag`）
- 過濾邏輯：`packages/functions/src/functions/api/item.rs`（`special_flag` 位掩碼）
- 歸檔分片：`packages/functions/src/functions/api/marker_doc.rs` / `item_doc.rs`

## 1. 爲什麼需要這麼多套標記

原神地圖不是「所有數據對所有人可見」這麼簡單，它至少要處理三種現實需求：

- **防劇透**：玩家還沒解鎖稻妻，地圖上就不該出現稻妻的雷神瞳位置；某些彩蛋類
  點位（比如隱藏成就觸發點）只該對主動選擇「看彩蛋」的玩家顯示。這是**可見性**
  問題。

- **測試服（Beta）數據隔離**：新版本上線前，編輯團隊要提前在地圖裏錄入下個版本
  （如楓丹、納塔）的點位，這些數據絕對不能漏給正式服玩家（會劇透 + 違規）。這
  是**數據隔離**問題。

- **物品查詢過濾**：地圖右上角的物品篩選面板裏，有些物品是「特殊物品」（比如活動
  限定、前臺默認不顯示的），玩家要勾選「顯示特殊物品」才查得出來。這是**查詢過濾**
  問題。

- **軟刪除**：點位下線後不能直接從庫裏抹掉（要保留審計軌跡、防止誤刪），但又不能
  再被任何查詢返回。這是**生命週期**問題。

`hidden_flag`、`special_flag`、`del_flag` 三套標記各自服務其中一類，且**互不耦合**：
一個點位的可見性、是否可查、是否已刪，是三個獨立的維度。下面分別展開。

## 2. hidden_flag：數據級可見性過濾

`HiddenFlag`（`packages/utils/src/types/common.rs`）是社區地圖特有的**數據級可見性**
枚舉，存爲 i32：

| 枚舉成員 | 數值 | 含義 | 誰能看 |
| --- | --- | --- | --- |
| `Visible` | `0` | 正式數據 | 所有玩家 |
| `Hidden` | `1` | 隱藏 | 僅測試 / 後臺 |
| `Beta` | `2` | 測試服（舊名 `Spy`，數值與線協議不變） | 測試服玩家 |
| `Suprise` | `3` | 彩蛋 | 主動開啓彩蛋的玩家 |

注意：這是**數據級的過濾**，不是接口級的權限校驗。它解決的不是「這個用戶能不能調
這個 API」，而是「這個用戶能在地圖上看到哪些點位」。前端通過請求頭 `userDataLevel`
（一個**位掩碼**）聲明自己有權看到哪些層級——比如普通玩家傳的掩碼裏只置了 `Visible`
位，測試客戶端會同時置 `Hidden` / `Beta` 位。

這套標記的兩個主要用途：

- **防劇透 / 彩蛋隔離**：玩家尚未解鎖的區域裏的祕密（比如須彌某些夢之樹寶箱）、
  藏得很深的彩蛋點位，標成 `Suprise`，默認不顯示，玩家主動勾選「顯示彩蛋」才拉取。
  這樣既保留了「全收集」工具的完整性，又不破壞探索樂趣。

- **測試數據隔離**：下個版本的新地區（楓丹廷、納塔部落等）點位在錄入階段標成
  `Beta`，正式服前端拉不到；版本上線當天把對應點位的 `hidden_flag` 改回 `Visible`，
  全服即時可見。這避免了「上線日臨時錄數據」的慌亂。

### 2.1 下沉到歸檔層

`hidden_flag` 不只是查詢時過濾，而是**在 BinaryMD5 歸檔分片時就作爲第一級分組鍵**
（見 [歸檔導出文檔·第 3.1 節](./binarymd5-archive-export.md)）。`marker_doc`
和 `item_doc` 都按 `hidden_flag` 分組各自壓縮成獨立的 gzip blob、各自一個 MD5。這意味著
普通玩家客戶端**根本不會收到** `Hidden`/`Beta`/`Suprise` 組的 MD5——不是查詢時過濾掉，
而是數據分片壓根不下發。這是比「查詢過濾」更強的隔離：哪怕前端有 bug 漏過濾，
數據也沒傳到客戶端。

### 2.2 實體字段位置

`hidden_flag` 出現在所有內容域實體上，且晉升時自動繼承：

- `marker.hidden_flag`（點位可見性）
- `item.hidden_flag`（物品可見性）
- `area.hidden_flag`（地區可見性）
- `marker_punctuate.hidden_flag`——打點建議的暫存表也帶這個字段（審核工作流已棄用，
  暫存表隨 schema 保留），晉升/導入時原樣帶進正式 `marker`。所以一個點位上線後就
  擁有提交者設定的可見性。

字段都加了 `#[sea_orm(indexed)]`，因爲歸檔分組查詢要按它過濾。

## 3. special_flag：物品/地區查詢的位掩碼過濾

`special_flag` 是一個 **i32 位掩碼**（item 上是 `Option<i32>`，area 上是 `i32`），
和 `hidden_flag` 完全是兩回事：它不控制「誰能看」，而是控制**物品篩選面板裏查不查
得出來**。

### 3.1 語義：低位第一位 = 前臺是否顯示

`item.rs` 模型註釋寫明（`packages/database/src/models/item/item.rs`）：

> 特殊物品標記
> 低位第一位: 前臺是否顯示

也就是說第 0 位（值 1）表示「這是前臺默認不顯示的特殊物品」。地圖客戶端右上角的
物品篩選面板裏，默認只列出 `special_flag = 0`（無特殊標記）的普通物品；玩家勾選
「顯示特殊物品」後，才會把含特殊位的物品也查出來。典型場景：活動限定物品、調試用
物品、稀有度極高的隱藏收集品。

### 3.2 查詢邏輯：Java `selectPageItemByCondition` 的對齊

`item.rs::do_get_list`（`packages/functions/src/functions/api/item.rs`）實現了這個
位掩碼過濾，嚴格對齊 Java 的 `selectPageItemByCondition` 自定義查詢：

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

兩種分支：

- **客戶端傳 `special_flag = 0`**（默認）：精確過濾 `special_flag = 0` 的物品，即
  「無任何特殊標記的普通物品」。這是物品篩選面板的默認行爲。

- **客戶端傳 `special_flag > 0`**（某位被置 1）：過濾 `(special_flag & param) != 0`，
  即「至少含有客戶端指定的某一位特殊標記」的物品。位與運算支持未來擴展更多位
  （第 1 位、第 2 位……各代表一種特殊屬性），客戶端按位組合查詢。

這個過濾和分頁、地區篩選、名稱模糊、類型篩選疊加在一起（`do_get_list` 同時支持
`area_id_list` / `name` / `type_id_list`），共同構成物品查詢 UI 的後端。

### 3.3 area 上的 special_flag

地區表（`area.rs`）也有 `special_flag`，語義同樣是「低位第一位：前臺是否顯示」。
用於在地區選擇樹裏隱藏某些調試地區或活動地區。

## 4. 三套標記的正交關係

這三個標記經常被新人搞混，但它們是**完全正交**的三個維度：

| 標記 | 類型 | 控制的維度 | 典型問題 |
| --- | --- | --- | --- |
| `hidden_flag` | 枚舉（0~3） | **誰能看**（數據級可見性） | 「測試服玩家才能看到這個點位」 |
| `special_flag` | i32 位掩碼 | **查不查得出來**（查詢過濾） | 「勾選特殊物品才篩得出來」 |
| `del_flag` | bool | **是否已刪**（生命週期） | 「這個點位下線了，任何查詢都不能返回」 |

一個點位可以同時：已經被軟刪（`del_flag=true`）、原本是測試服數據
（`hidden_flag=Beta`）、且是特殊物品（`special_flag=1`）。三者互不影響。

### 4.1 與軟刪除 del_flag 的正交關係

`del_flag` 是**生命週期標記**，由 `SafeEntityTrait`（見
[架構概覽](../guides/architecture.md#safeentitytrait-模式)）統一管理。它的規則
是：**被軟刪的點位，不論其 `hidden_flag` / `special_flag` 是什麼，都一律被排除**。

這是因爲 `SafeEntityTrait::find_safety()` 在所有查詢的根部都加了 `del_flag = false`
過濾——`marker_doc.rs` / `item_doc.rs` 裏查全量數據用的就是 `Entity::find_safety().all()`，
所以歸檔分片天然不含軟刪點位；`do_get_list` 等業務查詢也都走 `find_safety`，軟刪物品
查不到。換句話說：

- `hidden_flag` / `special_flag` 是**業務過濾**（在不同場景下選擇性地返回數據）；
- `del_flag` 是**存在性過濾**（數據「物理上」還在庫裏，但「邏輯上」已經不存在了）。

兩者疊加的語義是：「先排除已刪的，再在剩下的裏按可見性 / 特殊性過濾」。哪怕一個點位
的 `hidden_flag=Visible`（對所有人可見），一旦 `del_flag=true`，它就從所有查詢和歸檔
裏消失——這正是軟刪除該有的行爲。

## 5. 常見誤用與規避

- **把 `hidden_flag` 當權限校驗**：`hidden_flag` 是數據級過濾，不是安全邊界。真正
  的「誰能調測試服 API」要在路由層用 `ExtractAuthInfo` + 角色中間件做。`hidden_flag`
  只是保證「即便前端漏過濾，數據也不下發」（通過歸檔分片），不替代鑑權。

- **把 `special_flag` 當 `hidden_flag` 用**：`special_flag=1` 的物品在默認物品篩選裏
  查不到，但它的點位（如果 `hidden_flag=Visible`）仍然會出現在地圖上和歸檔裏。
  想真正隱藏一個點位，要改 `hidden_flag`。

- **晉升時漏帶 `hidden_flag`**：打點建議晉升爲正式點位時，必須把
  `punctuate.hidden_flag` 原樣寫進 `marker.hidden_flag`（打點審核工作流已棄用，該
  語義保留在歷史實現中）。如果漏掉，提交者標成 `Suprise` 的彩蛋點會變成全服可見，
  劇透就漏出去了。

## 6. 與 Java 實現的對齊

| Java（`genshin-map-cloud`） | Rust |
| --- | --- |
| `HiddenFlag` 枚舉（0~3） | `HiddenFlag` 枚舉（`Visible/Hidden/Beta/Suprise`，`Beta` 舊名 `Spy`） |
| `userDataLevel` 請求頭位掩碼 | 由前端按位組合，後端按 flag 分組歸檔 |
| `selectPageItemByCondition`（special_flag 位掩碼） | `item::do_get_list`（`bit_and` + `eq(0)` 兩分支） |
| `BaseEntity.del_flag`（MyBatis-Plus 邏輯刪除） | `SafeEntityTrait::find_safety`（`del_flag=false` 過濾） |

注意 Rust 枚舉裏彩蛋成員拼寫是 `Suprise`（少一個 `r`，
`packages/utils/src/types/common.rs`），這是爲了和 Java 側的歷史拼寫保持字節級一致
（避免反序列化對不上），屬有意爲之，
不是筆誤——改拼寫會破壞與 Java 數據庫存量數據的兼容。
