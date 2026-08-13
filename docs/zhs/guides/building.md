# 构建指南

> [← 返回索引](../README.md) · 架构请见 [架构概览](./architecture.md)

## 前置条件

- **Rust 工具链**：由仓库根 `rust-toolchain.toml` 锁定为 `stable`，并自带

`rustfmt`、`clippy` 组件。首次进入仓库时 `rustup` 会自动安装。

- **[`just`](https://github.com/casey/just)**：所有构建/测试/钩子命令的统一入口。
- **Docker**（仅本地调试需要）：用于一键拉起 Postgres、Redis、MinIO。
- **`celestia-devtools`**：提交规范与缓存守护工具，由 `just init` 引导安装。

## 常用命令

```bash
just init          # celestia-devtools init + cargo fetch（联网一次，之后离线）
just hooks         # 安装 commit-msg 钩子（强制 gitmoji）
just build         # 构建 router（release；release 配置 LTO + opt-level=z）
just build --dev   # 调试构建
just build --clean # 先 cargo clean 再构建
just check         # cargo check --workspace --all-targets
just run           # 运行 _router 二进制
just test          # 全工作区测试（--no-fail-fast）
just ci            # fmt-check + clippy + check + test（CI 等价）
just fmt           # cargo fmt + Markdown 格式化
```

实时调试可用文件监听守护：`just dev-watch packages -- just run`。

## `.env` 文件

在仓库根目录创建 `.env`，至少配置数据库连接。完整可用项：

```env
DB_HOST=127.0.0.1
DB_PORT=5432
DB_PASSWORD=genshin_map
# 可选：DB_USER / DB_NAME 默认为 genshin_map
```

> `DB_PASSWORD` 是必填项；其余在本地 docker-compose 环境下有合理默认值。

## 本地依赖（docker-compose）

仓库根目录的 `dev.compose.yml`（**仅用于本地，非生产**）一键启动三个服务：

| 服务 | 端口 | 凭据 |
| --- | --- | --- |
| Postgres | 5432 | 用户/密码/库名均为 `genshin_map` |
| Redis | 6379 | 无密码 |
| MinIO | 9000（S3）/ 9001（控制台） | `genshin_cloud` / `genshin_cloud` |

启动：`docker compose -f dev.compose.yml up -d`。

## CI（GitHub Actions）

`.github/workflows/` 下有三个工作流：

- **`rust.yml`**：构建与 `clippy` 检查。
- **`test.yml`**：`cargo test` 全工作区测试。
- **`docs.yml`**：文档构建/部署。

本地 `just ci` 即为 CI 的等价命令，提交前跑一遍可避免流水线红灯。
