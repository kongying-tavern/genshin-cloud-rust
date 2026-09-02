# Architecture

This guide describes how the four-package workspace fits together, how a
request flows from the network to PostgreSQL, the `SafeEntityTrait` pattern
that guards every write, and where Redis and MinIO plug in.

## Package layering

The workspace is a strict, four-layer dependency chain. Each crate is
re-exported under a `_`-prefixed alias so call sites stay readable.

```text
packages/
  utils/      ── shared utilities, DTO/VO types, SafeEntityTrait, jwt, bcrypt
  database/   ── sea-orm entities; depends on utils
  functions/  ── business logic; depends on database + utils
  router/     ── axum routes & middlewares, the _router binary; depends on all
```

| Crate | Role | Key contents |
| --- | --- | --- |
| `utils` (`_utils`) | Foundation. No knowledge of HTTP or business rules. | `db_operations::SafeEntityTrait` + `impl_safe_operation!` macro, `jwt::AuthInfo`, `bcrypt`, DTO/VO types under `src/models/` and `src/types/`. |
| `database` (`_database`) | Persistence. Holds the single `DB_CONN` global. | sea-orm entities grouped by domain (`models/{area,icon,item,marker,common,system}/`), `init_db_conn()` builds the `DatabaseConnectionMap { pg_conn, redis_conn, minio_conn }`. |
| `functions` (`_functions`) | Business logic. Pure async functions, no HTTP types. | `functions/api/<domain>.rs` (one file per content domain), `functions/system/{oauth,user}.rs`. |
| `router` (`_router`) | HTTP edge. The only crate that imports `axum`. | `routes/api/<domain>/` (one module per verb), `routes/system/`, `middlewares/{admin,auth,ip,role,user_agent}_extrator.rs`, `main.rs`. |

Because the lower crates never import `axum`, the business logic stays fully
testable without spinning up an HTTP server — the per-domain smoke tests in
`tests/rust/tests/<domain>/` exercise entities and functions directly.

## Request flow

A request traverses the stack top-to-bottom, then the response bubbles back up.

```text
HTTP request
   │
   ▼
axum Router (packages/router/src/main.rs)
   │  .layer(DefaultBodyLimit 16 MiB)
   │  .layer(ExtractIP)            ← middleware: parse client IP
   │  .layer(ExtractUserAgent)     ← middleware: parse UA
   ▼
routes::router()  →  /oauth/token | /system/* | /api/*
   │
   ▼
api::router()  nests each domain under /<domain>
   │  .layer(from_extractor::<ExtractAuthInfo>())  ← JWT → AuthInfo
   ▼
routes/api/<domain>/<verb>.rs   (axum handler)
   │  parses Json<T>, Query<T>, Path<T>
   ▼
_functions::functions::api::<domain>::do_*(auth, payload)
   │  validation, optimistic-lock bookkeeping
   ▼
_database::models::<domain>::Entity  (sea-orm)
   │  find_safety / update_safety / delete_safety
   ▼
DatabaseConnectionMap.pg_conn  →  PostgreSQL (schema: genshin_map)
```

The same handler shape repeats across every domain: an `axum` handler in
`routes/api/<domain>/<verb>.rs` extracts the request, delegates to a
`do_*` function in `functions/api/<domain>.rs`, and maps the `Result` into a
status code. The auth middleware (`ExtractAuthInfo`) runs once per `/api/*`
request and converts the `Authorization: Bearer <jwt>` header into an
`AuthInfo` value threaded through every business function.

## The SafeEntityTrait pattern

Every content entity (area, icon, item, marker, ...) carries two invariant
columns:

- `version: i64` — optimistic-lock counter, incremented on every update.
- `del_flag: bool` — soft-delete flag. Hard deletes are **rejected at the

sea-orm layer**: `before_save`/`before_delete` in the `impl_safe_operation!`
macro raises `DbErr` on any hard delete, so "delete" always means
`del_flag = true`.

The macro (in `packages/utils/src/db_operations.rs`) generates a
`SafeEntityTrait` impl that exposes `find_safety()`, `find_safety_by_id(id)`,
`update_safety(model)`, and `delete_safety(model)` / `delete_safety_by_id(id)`.
The "safety" variants:

1. `find_safety*` automatically appends `WHERE del_flag = false`.
1. `update_safety` re-reads the current `version`, increments it, and adds

`WHERE version = <old>` — a lost update silently updates zero rows instead
of clobbering data.

1. `delete_safety` flips `del_flag` to `true` (an `UPDATE`, not a `DELETE`).

This is the Rust equivalent of the Java `SafeSqlOperator` / mybatis-plus
optimistic-lock interceptor; business code in `functions/` should **never**
call the raw `Entity::find()` / `Entity::update()` / `Entity::delete()`.

## Redis and MinIO integration points

Both are provisioned in `database::init_db_conn()` and live on the same
`DatabaseConnectionMap` as the Postgres connection:

- **Redis** (`redis_conn`) — hot cache for the read-heavy front-end APIs.

It stores the OAuth session entries (`jwt:access:*` / `jwt:refresh:*`, which
power token lookup, kick-out, and revocation), backs the login rate limiter,
and holds the BinaryMD5 second-level result cache (`binmd5:result:*` with
epoch-based invalidation; see
[BinaryMD5 archive export](../designs/binarymd5-archive-export.md)). The
admin `cache` domain (`routes/api/cache/`) exposes DELETE endpoints that
flush these caches.

- **MinIO** (`minio_conn`) — S3-compatible object storage. On startup the

`images` bucket is created (if missing) with a public `s3:GetObject` policy
for serving uploaded icons and assets. The `bz2doc` bucket is provisioned
for future BinaryMD5 archive storage (currently the `*_doc` endpoints
generate GZIP-compressed blobs on-the-fly from PostgreSQL rather than
reading from MinIO).

The default local endpoints are `localhost:5432` (Postgres), `localhost:6379`
(Redis), and `http://localhost:9000` (MinIO) — see `dev.compose.yml` for the
matching docker-compose stack.
