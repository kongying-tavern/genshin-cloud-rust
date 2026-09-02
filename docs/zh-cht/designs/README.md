<!-- markdownlint-disable MD033 MD041 -->

# 設計文檔索引

> [← 返回總索引](../SUMMARY.md) · [架構概覽](../guides/architecture.md)

空熒酒館·原神地圖是一個**衆包貢獻型社區地圖**：神瞳、寶箱、奇饋寶箱、地籠、
採集物等點位主要由玩家貢獻，編輯團隊審核後才進入正式數據。這套「人人可打點、
人人可見、但需把關」的協作模型，催生了若干在普通增刪改查後端裏看不到的設計
決策。本目錄收錄這些決策的設計文檔，重點說明**爲什麼**在原神互動地圖的業務
語境下要這麼做，而不是把通用後端模板照搬過來。

每篇文檔大致包含三部分：背景與動機（這個數據結構 / 管線爲何存在）、對 Java
參考實現的對齊情況（`genshin-map-cloud`）、以及 Rust 側當前的落地狀態
（含已知簡化與後續待辦）。

## 文檔列表

| 文檔 | 主題 | 核心問題 |
| --- | --- | --- |
| [BinaryMD5 歸檔導出](./binarymd5-archive-export.md) | `*_doc` GZIP 壓縮批量導出管線 | 客戶端冷啓動如何快速拉取數千個 POI，且只同步變更頁 |
| [隱藏標記與特殊標記](./hidden-and-special-flags.md) | `hidden_flag` / `special_flag` / `del_flag` 三套正交標記 | 防劇透、測試服隔離、UI 過濾、軟刪除如何互不干擾 |

## 爲什麼單獨成文

這兩塊是整個 Rust 後端裏最「業務驅動」的部分，恰恰也是最容易在新人 review
時被誤改成「看起來更通用、實際破壞了遊戲語義」的部分：

- BinaryMD5 的「按 `id / 3000` 分頁 + MD5 尋址」看似是普通的分頁緩存，
  實則和客戶端的增量同步協議強綁定，改分頁粒度會同時動到前後端；
- `hidden_flag` 與 `special_flag` 名字相近但語義完全不同（一個是**誰能看**，
  一個是**查不查得出來**），且都和 `del_flag` 軟刪除正交，混用會直接把測試服
  數據漏給正式服玩家。

因此把它們從 [架構概覽](../guides/architecture.md) 裏獨立出來，作爲可被
鏈接、可被 review 引用的穩定參考。

## 相關資料

- Java 參考實現：[`genshin-map-cloud`](https://github.com/kongying-tavern/genshin-map-cloud)
- Rust 包結構：見 [架構概覽](../guides/architecture.md#四包分層) 的四包分層
- 領域術語對照（神瞳 / 寶箱 / 鋤地等）：見 [領域術語表](../guides/glossary.md)
- Java → Rust 同步進度：見 [Java 同步路線圖](../guides/sync-with-java-roadmap.md)
