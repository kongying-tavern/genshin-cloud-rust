<!-- markdownlint-disable MD033 MD041 MD036 -->
<h1 align="center">空荧酒馆·原神地图 Rust 后端</h1>
<h2 align="center">Genshin Map Cloud — Rust Backend</h2>

<p align="center"><strong>原神地图后端服务的 Rust 重写，与 Java 侧 (<code>java-genshin-map-cloud</code>) 功能对齐</strong></p>

<div align="center">

[![Rust](https://github.com/kongying-tavern/genshin-cloud-rust/workflows/Rust/badge.svg)](https://github.com/kongying-tavern/genshin-cloud-rust/actions/workflows/rust.yml)
[![Test](https://github.com/kongying-tavern/genshin-cloud-rust/workflows/Test/badge.svg)](https://github.com/kongying-tavern/genshin-cloud-rust/actions/workflows/test.yml)
[![GitHub](https://img.shields.io/badge/github-kongying--tavern%2Fgenshin--cloud--rust-blue.svg)](https://github.com/kongying-tavern/genshin-cloud-rust)

</div>

<div align="center">

**简体中文** · [English](./docs/en/guides/README.md)

</div>

---

## 简介 / Overview

本项目是「空荧酒馆·原神地图」后端的 Rust 实现，目标是与 Java 侧参考实现
([`java-genshin-map-cloud`](https://github.com/kongying-tavern/java-genshin-map-cloud))
的功能保持同步，并在性能、部署体验与类型安全上有所改进。

This project is the Rust rewrite of the "空荧酒馆 Genshin Map" backend. The goal
is feature parity with the Java reference implementation
([`java-genshin-map-cloud`](https://github.com/kongying-tavern/java-genshin-map-cloud))
while improving performance, deployment ergonomics, and type safety.

## 技术栈 / Tech Stack

| 层 / Layer | 技术 / Technology |
| --- | --- |
| Web 框架 / Framework | `axum` |
| ORM | `sea-orm` (PostgreSQL via `sqlx`) |
| 缓存 / Cache | `redis` |
| 对象存储 / Object storage | `minio` |
| 鉴权 / Auth | `jsonwebtoken` + `bcrypt` |
| 运行时 / Runtime | `tokio` |
| 日志 | `tracing` + `env_logger` |

## 快速开始 / Quick Start

在部署之前，请在项目根目录创建一个 `.env` 文件，存放数据库连接信息，示例：

Before deploying, create a `.env` file in the project root with the database
configuration, for example:

```env
DB_PASSWORD=<password>
```

正式开始前，请确保已安装 [`just`](https://github.com/casey/just) 与 `cargo`。
使用 `just dev` 启动开发栈（Rust 后端 + Vue3 前端）前，需要先在 `.env`
中配置 `E2E_VUE_FRONTEND` 指向本地 Vue3 前端项目的绝对路径（从
[kongying-tavern/`vue_map_register_v3`](https://github.com/kongying-tavern/`vue_map_register_v3`)
克隆）。

Before starting, make sure [`just`](https://github.com/casey/just) and `cargo`
are installed. To use `just dev` (Rust backend + Vue3 frontend dev stack), set
`E2E_VUE_FRONTEND` in `.env` to the absolute path of your local Vue3 frontend
project (cloned from
[kongying-tavern/`vue_map_register_v3`](https://github.com/kongying-tavern/`vue_map_register_v3`)).

```bash
just init          # 初始化开发环境 / initialize the dev environment
just hooks         # 安装 commit-msg 钩子 / install the commit-msg hook
just build         # 构建（release） / build (release)
just dev           # 启动开发栈（Rust + Vue）/ start dev stack (Rust + Vue)
just dev mock      # 启动 + Shirabe 浏览器 e2e 测试 / start + Shirabe e2e tests
just dev stop      # 停止 / stop
just dev status    # 状态 / check status
```

## 工作区结构 / Workspace Layout

```text
packages/
  utils/      # 通用工具、数据结构 / shared utilities & data structures
  database/   # 数据库实体与连接 / DB entities & connection
  functions/  # 业务逻辑层 / business logic
  router/     # axum 路由与中间件 / axum routes & middlewares
```

## 文档 / Documentation

完整架构、设计决策与指南位于 `docs/` 目录（多语言）：

Full architecture, design decisions, and guides live under `docs/`
(multilingual):

- **[简体中文文档](./docs/zhs/README.md)** — 架构、构建、API 参考、Java 同步路线图
- **[English docs](./docs/en/README.md)** — architecture, building, API

reference, Java-sync roadmap

## 提交规范 / Commit Convention

本项目遵循 celestia-island 组织的 [gitmoji 提交规范](https://gitmoji.dev)。
所有提交信息（subject line）必须为**英文**，以 gitmoji 开头、首字母大写、
以句号结尾。钩子会在 `git commit` 时强制校验。

This project follows the celestia-island org
[gitmoji convention](https://gitmoji.dev). All commit subjects must be in
**English**, starting with a gitmoji, capitalized, and ending with a period.
The commit-msg hook enforces this on every `git commit`.

详见 / See: [提交规范](./docs/zhs/guides/commit-message-convention.md) /
[Commit Convention](./docs/en/guides/commit-message-convention.md)

## 迭代工作流 / Iteration Workflow

`master` 是唯一主线，受分支保护：**任何补丁都必须以 PR 形式合入**。
从最新 master 切主题分支（`feat/`、`fix/`、`test/`、`docs/`、`refactor/`、
`chore/`），PR 标题同样遵循 gitmoji 规范（squash 合并时标题即合并提交），
CI 全绿后经 `celestia-devtools pr-merge`（或 `gh()` 代理函数）squash 合并，
合并后删除分支。历史 `dev` 分支已封存为 tag `archive/dev-snapshot`。

Master is the single protected mainline: **every patch lands via a PR**. Cut a
topic branch (`feat/`, `fix/`, `test/`, `docs/`, `refactor/`, `chore/`) from the
latest master, keep the PR title in the gitmoji format (it becomes the squash
merge subject), wait for green CI, then squash-merge via
`celestia-devtools pr-merge` (or the `gh()` proxy function) and delete the
branch. The historical `dev` branch is archived as tag `archive/dev-snapshot`.

迭代计划与未完成事项 backlog 见 / See [PLAN.md](./PLAN.md)。

## 许可证 / License

本项目为空荧酒馆（kongying-tavern）所有。详见仓库历史与上游 Java 项目。
