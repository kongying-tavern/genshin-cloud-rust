<!-- markdownlint-disable MD033 MD041 -->

# 空熒酒館·原神地圖 Rust 後端

> 頂層 README 的「簡體中文文檔」鏈接指向簡體版文檔索引；本頁爲繁體版[文檔索引](../README.md)中的詳細說明。
> [← 返回文檔索引](../README.md)

本項目是「空熒酒館·原神地圖」後端服務的 Rust 重寫，目標是與 Java 參考實現
（[`genshin-map-cloud`](https://github.com/kongying-tavern/genshin-map-cloud)）
功能對齊，並在性能、部署體驗與類型安全上有所改進。工作區按 `utils → database →
functions → router` 自底向上分層，逐層單向依賴。

## 技術棧

| 層 | 技術 |
| --- | --- |
| Web 框架 | `axum` 0.8（含 `macros`、`multipart`、`ws`） |
| ORM | `sea-orm` 2.x（PostgreSQL，經 `sqlx`，`runtime-tokio-rustls`） |
| 緩存 | `redis` 1.x |
| 對象存儲 | `minio` 0.4 |
| 鑑權 | `jsonwebtoken` 10 + `bcrypt` 0.19 |
| 運行時 | `tokio`（`full`） |
| 日誌 | `tracing` + `tracing-subscriber` |
| TLS | `rustls` 0.23 + `ring`（已剝離 `aws-lc-rs`） |

## 快速開始

前置工具：`just`、`cargo`（由 `rust-toolchain.toml` 鎖定的 stable Rust）、`docker`。

```bash
just init          # 初始化開發環境（celestia-devtools init + cargo fetch）
just hooks         # 安裝 commit-msg 鉤子（強制 gitmoji 規範）
just build         # 構建 router（release）
just build --dev   # 調試構建
just run           # 運行 router 二進制（_router）
just dev-watch packages -- just run   # 文件變動自動重啓
just test          # 全工作區測試
just ci            # fmt-check + clippy + check + test（CI 等價）
```

部署前在項目根目錄創建 `.env`，至少包含 `DB_PASSWORD`；詳見
[構建指南](./building.md)。

## 工作區結構

```text
packages/
  utils/      通用工具、DTO/VO、SafeEntityTrait 宏
  database/   sea-orm 實體（按域組織於 src/models/<domain>/）
  functions/  業務邏輯層（src/functions/api/<domain>.rs）
  router/     axum 路由與中間件（src/routes/api/<domain>/）
tests/rust/   端到端冒煙測試（按域組織）
```

## 提交規範

遵循 celestia-island 組織的 [gitmoji](https://gitmoji.dev) 規範：subject 必須爲
英文、以 gitmoji 開頭、首字母大寫、以句號結尾，不得使用 Conventional Commits
前綴（如 `feat:`）。`just hooks` 安裝的鉤子會在每次 `git commit` 時強制校驗。
詳見 [提交規範](./commit-message-convention.md)。

## 許可證

本項目爲空熒酒館（kongying-tavern）所有。具體條款見倉庫歷史與上游 Java 項目。
