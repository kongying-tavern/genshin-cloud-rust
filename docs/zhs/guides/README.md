<!-- markdownlint-disable MD033 MD041 -->

# 空荧酒馆·原神地图 Rust 后端

> 顶层 README 的「[简体中文文档](../../../README.md)」链接指向本页。
> [← 返回文档索引](../README.md)

本项目是「空荧酒馆·原神地图」后端服务的 Rust 重写，目标是与 Java 参考实现
（[`java-genshin-map-cloud`](https://github.com/kongying-tavern/java-genshin-map-cloud)）
功能对齐，并在性能、部署体验与类型安全上有所改进。工作区按 `utils → database →
functions → router` 自底向上分层，逐层单向依赖。

## 技术栈

| 层 | 技术 |
| --- | --- |
| Web 框架 | `axum` 0.8（含 `macros`、`multipart`、`ws`） |
| ORM | `sea-orm` 1.x（PostgreSQL，经 `sqlx`，`runtime-tokio-rustls`） |
| 缓存 | `redis` 1.x |
| 对象存储 | `minio` 0.3 |
| 鉴权 | `jsonwebtoken` 10 + `bcrypt` 0.19 |
| 运行时 | `tokio`（`full`） |
| 日志 | `tracing` + `tracing-subscriber` |
| TLS | `rustls` 0.23 + `ring`（已剥离 `aws-lc-rs`） |

## 快速开始

前置工具：`just`、`cargo`（由 `rust-toolchain.toml` 锁定的 stable Rust）、`docker`。

```bash
just init          # 初始化开发环境（celestia-devtools init + cargo fetch）
just hooks         # 安装 commit-msg 钩子（强制 gitmoji 规范）
just build         # 构建 router（release）
just build --dev   # 调试构建
just run           # 运行 router 二进制（_router）
just dev-watch packages -- just run   # 文件变动自动重启
just test          # 全工作区测试
just ci            # fmt-check + clippy + check + test（CI 等价）
```

部署前在项目根目录创建 `.env`，至少包含 `DB_PASSWORD`；详见
[构建指南](./building.md)。

## 工作区结构

```text
packages/
  utils/      通用工具、DTO/VO、SafeEntityTrait 宏
  database/   sea-orm 实体（按域组织于 src/models/<domain>/）
  functions/  业务逻辑层（src/functions/api/<domain>.rs）
  router/     axum 路由与中间件（src/routes/api/<domain>/）
tests/rust/   端到端冒烟测试（按域组织）
```

## 提交规范

遵循 celestia-island 组织的 [gitmoji](https://gitmoji.dev) 规范：subject 必须为
英文、以 gitmoji 开头、首字母大写、以句号结尾，不得使用 Conventional Commits
前缀（如 `feat:`）。`just hooks` 安装的钩子会在每次 `git commit` 时强制校验。
详见 [提交规范](./commit-message-convention.md)。

## 许可证

本项目为空荧酒馆（kongying-tavern）所有。具体条款见仓库历史与上游 Java 项目。
