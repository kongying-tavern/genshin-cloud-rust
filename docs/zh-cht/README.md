<!-- markdownlint-disable MD033 MD041 -->

# 空熒酒館·原神地圖 Rust 後端

> 繁體中文文檔索引（English、簡體中文版本見頁面底部語言切換菜單）

本項目是「空熒酒館·原神地圖」後端服務的 Rust 實現，目標是與 Java 參考實現
（[`genshin-map-cloud`](https://github.com/kongying-tavern/genshin-map-cloud)）
保持功能同步，同時在性能、部署體驗與類型安全上有所改進。

技術棧基於 `axum` + `sea-orm` + `PostgreSQL` + `redis` + `minio`，工作區劃分爲
`utils / database / functions / router` 四個包，逐層從底向上依賴。鑑權由
`jsonwebtoken` + `bcrypt` 提供，運行時爲 `tokio`，日誌走 `tracing`。

## 指南 / Guides

| 文檔 | 說明 |
| --- | --- |
| [詳細 README](./guides/README.md) | 項目概述、技術棧、快速開始 |
| [領域術語表](./guides/glossary.md) | 中英文領域術語對照 |
| [架構概覽](./guides/architecture.md) | 四包分層、請求流、`SafeEntityTrait`（樂觀鎖 + 軟刪除）與緩存集成點 |
| [構建指南](./guides/building.md) | 前置工具鏈、`just` 命令、`.env`、本地 `docker-compose` 與 CI |
| [API 參考](./guides/api-reference.md) | router 暴露的全部 API 域，按用途分組 |
| [提交規範](./guides/commit-message-convention.md) | celestia-devtools gitmoji 規範、鉤子安裝與跳過方式 |
| [Java 同步路線圖](./guides/sync-with-java-roadmap.md) | Java 側範圍與七個移植優先級批次 |
| [域同步模板](./guides/domain-sync-template.md) | 單域移植的五層落地模式與 area 示例 |

## 設計文檔 / Designs

- [BinaryMD5 歸檔導出](./designs/binarymd5-archive-export.md) — 客戶端冷啓動的全量數據 GZIP 壓縮增量同步管線
- [隱藏標記與特殊標記](./designs/hidden-and-special-flags.md) — `hidden_flag` 數據級防劇透 + `special_flag` 位掩碼過濾

以下設計文檔已編寫完成。後續將記錄 `rustls+ring` 加密後端選型、`sea-orm` 1.x→2.x
遷移、`SafeEntityTrait` 宏重寫等關鍵決策。

## 快速入口

- 完整 README（含快速開始與許可證）：[詳細說明](./guides/README.md)
- 目錄（mdBook/lagrange 風格）：[SUMMARY](./SUMMARY.md)
- 頂層項目說明：[倉庫根 README](../../README.md)

## 其他語言 / Other Languages

本站提供 English / 简体中文 / 繁體中文 三種語言，請使用頁面底部的語言
切換菜單（偏好會保存在瀏覽器中，連結可透過 `?lang=` 參數分享）。
