<!-- markdownlint-disable MD033 MD041 MD036 -->
<h1 align="center">空荧酒馆·原神地图 Rust 后端</h1>
<h2 align="center">Genshin Map Cloud — Rust Backend</h2>

<p align="center"><strong>原神地图后端服务的 Rust 重写，与 Java 侧 (<code>genshin-map-cloud</code>) 功能对齐</strong></p>

<div align="center">

[![Rust](https://github.com/kongying-tavern/genshin-cloud-rust/workflows/Rust/badge.svg)](https://github.com/kongying-tavern/genshin-cloud-rust/actions/workflows/rust.yml)
[![Test](https://github.com/kongying-tavern/genshin-cloud-rust/workflows/Test/badge.svg)](https://github.com/kongying-tavern/genshin-cloud-rust/actions/workflows/test.yml)
[![GitHub](https://img.shields.io/badge/github-kongying--tavern%2Fgenshin--cloud--rust-blue.svg)](https://github.com/kongying-tavern/genshin-cloud-rust)

</div>

<div align="center">

**简体中文** · [English](./docs/en/README.md)

</div>

---

## 简介 / Overview

本项目是「空荧酒馆·原神地图」后端的 Rust 实现，目标是与 Java 侧参考实现
([`genshin-map-cloud`](https://github.com/kongying-tavern/genshin-map-cloud))
的功能保持同步，并在性能、部署体验与类型安全上有所改进。

This project is the Rust rewrite of the "空荧酒馆 Genshin Map" backend. The goal
is feature parity with the Java reference implementation
([`genshin-map-cloud`](https://github.com/kongying-tavern/genshin-map-cloud))
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

在部署之前，请在项目根目录创建一个 `.env` 文件，存放数据库连接信息与 JWT
密钥，示例：

Before deploying, create a `.env` file in the project root with the database
configuration and the JWT secret, for example:

```env
DB_PASSWORD=<password>
JWT_SECRET=<output of: openssl rand -base64 48>
```

完整变量清单见下方 [环境变量](#环境变量--environment-variables) 章节。

See the [Environment Variables](#环境变量--environment-variables) section below
for the complete list.

正式开始前，请确保已安装 [`just`](https://github.com/casey/just)、`cargo` 与
`celestia-devtools`（见下方[开发工具链](#开发工具链--toolchain)）。使用
`just dev` 启动开发栈（Rust 后端 + Vue3 前端）前，需要先在 `.env` 中配置
`E2E_VUE_FRONTEND` 指向本地 Vue3 前端项目的绝对路径（从
[kongying-tavern/map_register_v3](https://github.com/kongying-tavern/map_register_v3)
克隆）。

Before starting, make sure [`just`](https://github.com/casey/just), `cargo`,
and `celestia-devtools` are installed (see
[Toolchain](#开发工具链--toolchain) below). To use `just dev` (Rust backend +
Vue3 frontend dev stack), set `E2E_VUE_FRONTEND` in `.env` to the absolute path
of your local Vue3 frontend project (cloned from
[kongying-tavern/map_register_v3](https://github.com/kongying-tavern/map_register_v3)).

```bash
just init          # 初始化开发环境 / initialize the dev environment
just hooks         # 安装 commit-msg 钩子 / install the commit-msg hook
just build         # 构建（release） / build (release)
just dev           # 启动开发栈（Rust + Vue）/ start dev stack (Rust + Vue)
just dev mock      # 启动 + Shirabe 浏览器 e2e 测试 / start + Shirabe e2e tests
just dev stop      # 停止 / stop
just dev status    # 状态 / check status
```

## 开发工具链 / Toolchain

本仓库的开发辅助命令（gitmoji commit-msg 钩子、Markdown 格式化、cargo 缓存
守护、PR 合并校验等）来自
[`celestia-island/celestia-devtools`](https://github.com/celestia-island/celestia-devtools)。
`just init` 会引导安装；仓库根目录的 `celestia-devtools.just` 是其 vendored
裁剪版（刷新：`celestia-devtools init --force`）。

安装（每台机器一次）：

```bash
pip install git+https://github.com/celestia-island/celestia-devtools.git
# 或：clone 仓库后在仓库内执行 pip install -e .
```

`pr-merge`、`gh()` 代理函数、CI workflow 等用法见其 README。

The dev tooling in this repo (gitmoji commit-msg hook, Markdown formatting,
cargo cache guard, PR-merge validation, …) comes from
[`celestia-island/celestia-devtools`](https://github.com/celestia-island/celestia-devtools).
`just init` bootstraps it; the vendored, trimmed copy lives at
`celestia-devtools.just` in the repo root (refresh with
`celestia-devtools init --force`).

Install once per machine:

```bash
pip install git+https://github.com/celestia-island/celestia-devtools.git
# or: clone the repo and run: pip install -e .
```

See its README for `pr-merge`, the `gh()` proxy function, and CI workflow
usage.

## 前端联动 / Frontend Integration

本后端实现与 Java 侧一致的 API 契约，直接服务空荧酒馆的前端项目（绝大多数
`/api/*` 路由已逐条对齐；Java 的 `route` / `punctuate` / `punctuate_audit`
域**有意不提供**——见 `docs/zh-chs/guides/sync-with-java-roadmap.md`）：

| 前端仓库 | 关系 |
| --- | --- |
| [kongying-tavern/map_front_v3](https://github.com/kongying-tavern/map_front_v3)（空荧酒馆前端地图 v3，线上 v3.yuanshen.site） | **生产前端**：按 API 契约消费本后端，后端语言切换（Java → Rust）对前端透明。 |
| [kongying-tavern/map_register_v3](https://github.com/kongying-tavern/map_register_v3)（空荧后厨·地图数据综合管理应用） | **管理前端**：`just dev` 通过 `E2E_VUE_FRONTEND` 指向其本地克隆与本后端联调；图片上传走 `/api/res/upload/image`。 |

另外，`/cdn/*` 反向代理把前端静态资源透传给浏览器：默认上游
`https://v3.yuanshen.site`（可用 `CDN_UPSTREAM` 覆盖），图标经
`/cdn/img-proxy` 白名单转发，dadian 配置由 `CDN_DADIAN_CONFIG` 提供本地
兜底（`/cdn/dadian-preview.json.bz2`）。

This backend implements the same API contract as the Java side and serves the
Kongying Tavern frontend projects directly (the vast majority of `/api/*`
routes are aligned one by one; the Java `route` / `punctuate` /
`punctuate_audit` domains are **intentionally not provided** — see
`docs/zh-chs/guides/sync-with-java-roadmap.md`):

| Frontend repo | Relationship |
| --- | --- |
| [kongying-tavern/map_front_v3](https://github.com/kongying-tavern/map_front_v3) (the v3 map frontend, live at v3.yuanshen.site) | **Production frontend**: consumes this backend through the API contract; switching the backend language (Java → Rust) is transparent to it. |
| [kongying-tavern/map_register_v3](https://github.com/kongying-tavern/map_register_v3) (空荧后厨, the map data management app) | **Admin frontend**: `just dev` points `E2E_VUE_FRONTEND` at its local clone for joint development; image uploads go through `/api/res/upload/image`. |

The `/cdn/*` reverse proxy also forwards frontend static assets to browsers:
the upstream defaults to `https://v3.yuanshen.site` (override with
`CDN_UPSTREAM`), icons pass through the whitelisted `/cdn/img-proxy`, and the
dadian config is served locally from `CDN_DADIAN_CONFIG`
(`/cdn/dadian-preview.json.bz2`).

## 环境变量 / Environment Variables

所有变量均从进程环境或根目录 `.env` 读取（模板见 `.env.example`）。

All variables are read from the process environment or the root `.env` file
(template: `.env.example`).

| 变量 / Variable | 必填 / Required | 默认 / Default | 用途与用法 / Purpose & usage |
| --- | --- | --- | --- |
| `JWT_SECRET` | **是** / **Yes** | 无 / none | HS256 签名密钥，缺失直接拒绝启动。生成：`openssl rand -base64 48`。<br>HS256 signing secret; the process refuses to start without it. Generate with `openssl rand -base64 48`. |
| `DB_HOST` | 否 / No | `localhost` | PostgreSQL 主机。/ PostgreSQL host. |
| `DB_PORT` | 否 / No | `5432` | PostgreSQL 端口；非法值启动报错。/ PostgreSQL port; invalid values fail startup. |
| `DB_USERNAME` | 否 / No | `genshin_map` | 数据库用户名。/ Database username. |
| `DB_PASSWORD` | 否* / No* | 空 / empty | 数据库密码，可含 `@` `:` 等特殊字符（自动百分号转义）；*生产环境必填。/ Database password; reserved characters (`@`, `:`, …) are percent-encoded automatically. *Required in production. |
| `DB_DATABASE` | 否 / No | `genshin_map` | 数据库名。/ Database name. |
| `DB_SCHEMA` | 否 / No | `genshin_map` | 表所在 schema（经连接 `search_path` 生效）。必须是纯 SQL 标识符，否则回退默认并告警。/ Schema for all tables (via the connection `search_path`). Must be a bare SQL identifier, otherwise the default is used with a warning. |
| `PORT` | 否 / No | `80` | HTTP 监听端口。/ HTTP listen port. |
| `APP_ENV` | 否 / No | `dev` | 运行环境标识，写入 `/oauth/token` 登录响应的 `env` 字段（Java active profile 契约；生产部署设 `prod`）。/ Environment identifier emitted as the `env` field of the `/oauth/token` login response (Java active-profile contract; set `prod` in production). |
| `LOG_DIR` | 否 / No | 未设置 / unset | 设置后日志同时追加写入 `<LOG_DIR>/genshin-cloud.log`（目录不存在时自动创建，stderr 输出保留）。/ When set, logs are also appended to `<LOG_DIR>/genshin-cloud.log` (the directory is created if missing; stderr output stays on). |
| `REDIS_HOST` | 否 / No | `localhost` | Redis 主机；未配置/不可达时后端降级运行。/ Redis host; the backend degrades gracefully when unreachable. |
| `REDIS_PORT` | 否 / No | `6379` | Redis 端口。/ Redis port. |
| `REDIS_USERNAME` | 否 / No | 空 / empty | Redis 用户名。/ Redis username. |
| `REDIS_PASSWORD` | 否 / No | 空 / empty | Redis 密码。/ Redis password. |
| `REDIS_REQUIRED` | 否 / No | `false` | `true` 时 Redis 不可达将拒绝依赖 Redis 的鉴权路径（fail-closed），而非降级。/ When `true`, Redis-dependent auth paths fail closed instead of degrading. |
| `MINIO_BASE_URL` | 否 / No | `http://localhost:9000` | 后端**内部**上传使用的 MinIO 地址。/ MinIO address used internally for uploads. |
| `MINIO_ACCESS_KEY` / `MINIO_SECRET_KEY` | 否 / No | 未设置 / unset | 未设置时跳过 MinIO，上传接口显式报错。/ When unset, MinIO is skipped and upload endpoints fail explicitly. |
| `MINIO_PUBLIC_BASE_URL` | 否 / No | `MINIO_BASE_URL` | 返回给客户端的文件 URL 使用的公网地址（CDN/反代改写地址时设置）。/ Public base for file URLs returned to clients; set it when a CDN/reverse proxy rewrites the public address. |
| `CDN_UPSTREAM` | 否 / No | `https://v3.yuanshen.site` | `/cdn` 反代上游。/ Upstream for the `/cdn` reverse proxy. |
| `CDN_DADIAN_CONFIG` | 否 / No | 内置空配置 / built-in empty config | 本地预生成 dadian 配置（bz2）路径，服务 `/cdn/dadian-preview.json.bz2`。/ Path of a locally pre-generated dadian config (bz2) served at `/cdn/dadian-preview.json.bz2`. |
| `ICON_PROXY_BASE` | 否 / No | 未设置 / unset | 设置后 `tag_doc` 把 ddns.minemc.top 图标改写为经本后端 `/cdn/img-proxy` 的地址（指向后端对外地址）。/ When set, `tag_doc` rewrites ddns.minemc.top icon URLs to go through this backend's `/cdn/img-proxy` (point it at the backend's public address). |
| `CORS_ALLOW_ORIGIN` | 否 / No | 未设置 / unset | 逗号分隔的浏览器跨域白名单；未设置时不发 CORS 头（同源/Vite 代理仍可用）。/ Comma-separated browser-origin allowlist; when unset no CORS headers are sent (same-origin/Vite proxy still works). |
| `JWT_RSA_PRIVATE_KEY_PEM` | 否 / No | 未设置 / unset | 设置后 token 改用 RS256 签名，`/.well-known/jwks.json` 发布 RSA 公钥。/ When set, tokens are signed with RS256 and the JWKS endpoint publishes the RSA public key. |
| `JWT_RSA_VERIFY_KEYS` | 否 / No | 空 / empty | 轮换期的历史 RSA 公钥 PEM（逗号分隔），旧 token 保持可验证。/ Historical RSA public-key PEMs (comma-separated) kept verifiable during rotation. |
| `JWT_ACCEPT_HS256` | 否 / No | `false` | RS256 模式下是否仍接受 HS256 token（**仅本地调试**，线上禁止）。/ Whether HS256 tokens stay accepted in RS256 mode (**local/dev only**; never enable in production). |
| `SKIP_ACCESS_POLICY` | 否 / No | `false` | `true` 跳过 IP/UA 访问策略校验（开发期）。/ `true` skips the IP/User-Agent access-policy checks (dev only). |
| `TRUST_PROXY_HEADERS` | 否 / No | 未设置 / unset | 设置后信任反代的 `X-Real-IP` / `X-Forwarded-For`（nginx 反代后必须，否则访问策略与审计日志拿到反代地址）。/ When set, trust `X-Real-IP` / `X-Forwarded-For` from the reverse proxy (required behind nginx; otherwise clients could spoof their IP). |
| `INIT_ADMIN_USERNAME` / `INIT_ADMIN_PASSWORD` | 否 / No | `admin` / `admin123` | 仅 `cargo run --bin init_db` 首次播种开发者管理员时使用。/ Only used by `cargo run --bin init_db` when seeding the dev admin account. |
| `E2E_VUE_FRONTEND` | 仅 `just dev` / `just dev` only | 无 / none | 本地 map_register_v3 前端项目绝对路径。/ Absolute path to the local map_register_v3 frontend project. |
| `E2E_USERNAME` / `E2E_PASSWORD` | 否 / No | 空 / empty | e2e 已鉴权断言使用的真实账号；未设置时对应测试**显式跳过**。/ A real backend account for authenticated e2e assertions; without it those tests SKIP explicitly. |
| `GCS_TEST_DB` | 否（仅测试）/ No (tests only) | 未设置 / unset | 设置后启用需要真实数据库的集成测试。/ Enables the integration tests that need a real database. |

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

- **[简体中文文档](./docs/zh-chs/README.md)** — 架构、构建、API 参考、Java 同步路线图
- **[English docs](./docs/en/README.md)** — architecture, building, API reference, Java-sync roadmap

## 提交规范 / Commit Convention

本项目遵循 celestia-island 组织的 [gitmoji 提交规范](https://gitmoji.dev)。
所有提交信息（subject line）必须为**英文**，以 gitmoji 开头、首字母大写、
以句号结尾。钩子由
[`celestia-devtools`](https://github.com/celestia-island/celestia-devtools)
在 `git commit` 时强制校验。

This project follows the celestia-island org
[gitmoji convention](https://gitmoji.dev). All commit subjects must be in
**English**, starting with a gitmoji, capitalized, and ending with a period.
The [`celestia-devtools`](https://github.com/celestia-island/celestia-devtools)
commit-msg hook enforces this on every `git commit`.

详见 / See: [提交规范](./docs/zh-chs/guides/commit-message-convention.md) /
[Commit Convention](./docs/en/guides/commit-message-convention.md)

## 迭代工作流 / Iteration Workflow

`master` 是唯一主线，受分支保护：**任何补丁都必须以 PR 形式合入**。
从最新 master 切主题分支（`feat/`、`fix/`、`test/`、`docs/`、`refactor/`、
`chore/`），PR 标题同样遵循 gitmoji 规范（squash 合并后 PR 标题即合并提交的主题），
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
