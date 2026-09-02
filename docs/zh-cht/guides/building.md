# 構建指南

> [← 返回索引](../README.md) · 架構請見 [架構概覽](./architecture.md)

## 前置條件

- **Rust 工具鏈**：由倉庫根 `rust-toolchain.toml` 鎖定爲 `stable`，並自帶
  `rustfmt`、`clippy` 組件。首次進入倉庫時 `rustup` 會自動安裝。

- **[`just`](https://github.com/casey/just)**：所有構建/測試/鉤子命令的統一入口。
- **Docker**（僅本地調試需要）：用於一鍵拉起 Postgres、Redis、MinIO。
- **`celestia-devtools`**：提交規範與緩存守護工具，來自
  [celestia-island/celestia-devtools](https://github.com/celestia-island/celestia-devtools)，
  安裝：`pip install git+https://github.com/celestia-island/celestia-devtools.git`
  （也可 clone 後 `pip install -e .`）。倉庫根的 `celestia-devtools.just` 是
  vendored 裁剪版，用 `celestia-devtools init --force` 刷新。

## 常用命令

```bash
just init          # celestia-devtools init + cargo fetch（聯網一次，之後離線）
just hooks         # 安裝 commit-msg 鉤子（強制 gitmoji）
just build         # 構建 router（release；release 配置 LTO + opt-level=z）
just build --dev   # 調試構建
just build --clean # 先 cargo clean 再構建
just check         # cargo check --workspace --all-targets
just run           # 運行 _router 二進制
just test          # 全工作區測試（--no-fail-fast）
just ci            # fmt-check + clippy + check + test（CI 等價）
just fmt           # cargo fmt + Markdown 格式化
```

實時調試可用文件監聽守護：`just dev-watch packages -- just run`。

## `.env` 文件

在倉庫根目錄創建 `.env`，至少配置數據庫連接。完整可用項：

```env
DB_HOST=127.0.0.1
DB_PORT=5432
DB_USERNAME=genshin_map
DB_PASSWORD=genshin_map
DB_DATABASE=genshin_map
JWT_SECRET=<openssl rand -base64 48>
```

> `JWT_SECRET` 爲必填項（進程啓動時校驗，缺失即拒絕啓動）；其餘 DB 項在
> 本地 docker-compose 環境下有合理默認值。完整變量清單與默認值見根
> `README.md` 的「環境變量」章節與 `.env.example`。

## 本地依賴（docker-compose）

倉庫根目錄的 `dev.compose.yml`（**僅用於本地，非生產**）一鍵啓動三個服務：

| 服務 | 端口 | 憑據 |
| --- | --- | --- |
| Postgres | 5432 | 用戶/密碼/庫名均爲 `genshin_map` |
| Redis | 6379 | 無密碼 |
| MinIO | 9000（S3）/ 9001（控制檯） | `genshin_cloud` / `genshin_cloud` |

啓動：`docker compose -f dev.compose.yml up -d`。

## CI（GitHub Actions）

`.github/workflows/` 下有八個工作流：

- **`rust.yml`**：`cargo fmt --check` / `check` / `clippy -D warnings` /
  `build --release`，外加按根 `Cargo.toml` 聲明的 `rust-version = "1.94"` 做 MSRV 檢查（下限來自 sqlx 0.9）。
- **`test.yml`**：`cargo test` 全工作區測試（ubuntu/windows 矩陣）、Postgres
  DB 集成測試、`trufflehog` 密鑰掃描。
- **`commit-msg.yml`**：PR 標題與每個提交主題的 gitmoji 規範 lint。
- **`deny.yml`**：cargo-deny 依賴安全/許可證/來源檢查，另每週定時掃新公告。
- **`hygiene.yml`**：actionlint（workflow 語法）+ shellcheck（shell 腳本）。
- **`coverage.yml`**：cargo-llvm-cov 覆蓋率報告（僅供參考，不做閾值門禁）。
- **`docker.yml`**：構建並推送 GHCR 生產鏡像。
- **`docs.yml`**：用 lagrange 構建多語言文檔並部署到 GitHub Pages。

本地 `just ci` 即爲 CI 的等價命令，提交前跑一遍可避免流水線紅燈。
