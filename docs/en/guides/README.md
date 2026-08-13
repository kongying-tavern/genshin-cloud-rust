# Genshin Map Cloud — Rust Backend

The Rust rewrite of the "空荧酒馆 Genshin Map" backend. The goal is **feature
parity** with the Java reference implementation
([`java-genshin-map-cloud`](https://github.com/kongying-tavern/java-genshin-map-cloud))
while improving performance, deployment ergonomics, and type safety. The Rust
and Java sides share the same PostgreSQL schema (`genshin_map`) during the
migration, so they can run against the same database.

> **简体中文** · [English](./README.md) — this is the English entry point,
> linked from the top-level [`README.md`](../../../README.md).

## Tech stack

| Layer | Technology |
| --- | --- |
| Web framework | [`axum`](https://crates.io/crates/axum) 0.8 (macros, json, query, multipart, ws) |
| ORM | [`sea-orm`](https://crates.io/crates/sea-orm) 1.x over PostgreSQL via `sqlx` |
| Cache | [`redis`](https://crates.io/crates/redis) 1.x (tokio runtime) |
| Object storage | [`minio`](https://crates.io/crates/minio) 0.3 (S3-compatible) |
| Auth | [`jsonwebtoken`](https://crates.io/crates/jsonwebtoken) 10 + [`bcrypt`](https://crates.io/crates/bcrypt) 0.19 |
| Runtime | [`tokio`](https://crates.io/crates/tokio) 1.x (multi-thread) |
| Logging | [`tracing`](https://crates.io/crates/tracing) + `tracing-subscriber`, `env_logger` shim |
| TLS | `rustls` 0.23 with the `ring` provider only (no `aws-lc-rs` / `aws-lc-sys`) |

## Quick start

Before starting, install [`just`](https://github.com/casey/just), `cargo`, and
`docker`. Local debugging additionally needs `docker-compose`. The toolchain
is pinned by `rust-toolchain.toml` (stable + rustfmt + clippy).

```bash
just init          # initialize the dev environment (cargo fetch + devtools)
just hooks         # install the celestia-devtools commit-msg hook
just build         # build the workspace (release; add --dev for debug)
just dev           # start dev stack (Rust + Vue)
```

Create a `.env` in the repo root (see [Building](./building.md) for the full
variable list):

```env
DB_PASSWORD=genshin_map
```

## Workspace layout

```text
packages/
  utils/      # shared utilities, DTO/VO types, SafeEntityTrait, jwt/bcrypt
  database/   # sea-orm entities + the DB_CONN (Postgres + Redis + MinIO) map
  functions/  # business logic (functions/api/<domain>.rs, functions/system/*)
  router/     # axum routes & middlewares, the _router binary entry point
tests/
  rust/tests/<domain>/   # per-domain smoke tests (no DB connection needed)
```

Dependencies flow strictly downward: `router → functions → database → utils`.
Each lower crate is re-exported under a `_`-prefixed alias (`_utils`,
`_database`, `_functions`) so call sites read as `_functions::functions::api::area::do_add(...)`.

## Commit convention

This project enforces the `celestia-devtools` gitmoji convention via a
commit-msg hook. Every commit subject must be **English**, start with a
gitmoji, be capitalized, and end with a period — no Conventional Commits
prefixes (`feat:`, `fix:`). See
[Commit Convention](./commit-message-convention.md) for the full rules and
the skip override.

## License

Owned by 空荧酒馆 (kongying-tavern). See the repository history and the
upstream Java project for details.
