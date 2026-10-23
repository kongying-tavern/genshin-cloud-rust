# Java Sync Roadmap

This roadmap tracks the port of features from the Java reference backend
([`genshin-map-cloud`](https://github.com/kongying-tavern/genshin-map-cloud))
into this Rust rewrite. The Rust and Java sides share the same PostgreSQL
schema (`genshin_map`), verified by the table-name parity tests in
`tests/rust/tests/`, so the two backends can run against the same database
during the migration.

## Java-side scope

The Java backend is roughly **~30 controllers** and **~20 entities** across
two domains, plus three pieces of non-trivial infrastructure:

- **Map content**: area, icon, `icon_type`, item, `item_type`, marker,

`marker_link`, tag, `tag_type`, route, notice, history, score.

- **Binary archive export** (`*_doc` endpoints): large datasets are serialized,

GZIP-compressed, and keyed by BinaryMD5 so the client can incrementally sync.
Java caches them in Caffeine; the Rust port uses a two-tier cache (in-process
`moka` plus a Redis second level).

- **Crowd-sourced punctuate workflow**: user marker submissions → staging

table → audit → promotion to live markers (three-state approval).

- **System**: user, role, device (login-anomaly detection), invitation,

`action_log`, archive.

- **Auth**: OAuth2 password-grant JWT with a JWKS endpoint, RSA keypair,

device/IP anomaly detection on token issuance.

## Porting priority

The order below front-loads the entities that unblock the most downstream
work. Each step lists the key entity/feature, an estimated complexity, and the
current status.

| # | Area | Key entity / feature | Complexity | Status |
| --- | --- | --- | --- | --- |
| 1 | **area + marker** (reference samples) | `Area`, `Marker`, `hiddenFlag`, `specialFlag`. Establishes the `SafeEntityTrait` pattern and the five-layer domain template every later port copies. | Medium | **Done** — used as the porting template |
| 2 | **icon / item / tag families** | `Icon`, `IconType`, `IconTypeLink`, `Item`, `ItemType`, `ItemTypeLink`, plus the `Tag` / `TagType` taxonomy. Includes the `selectPageItemByCondition` `specialFlag` filter and copy/join/move_type. | Low–Medium | **Done** (covered by the `api_db` integration tests) |
| 3 | **notice / history (route excluded)** | `Notice` (validity-sort rule), `History`. The Java `Route` domain is deliberately not ported: the `route` entity stays in `models/common/` for schema parity, but the `/api/route` endpoints are not provided. | Low–Medium | **Done** |
| 4 | **punctuate workflow + scoring** | `MarkerPunctuate` staging → `Marker` promotion (three-state audit, role-gated and transactional) and `ScoreStat` aggregation (field-weighted). | High | **Punctuate workflow deprecated** (the staging table stays for schema parity); **scoring done** |
| 5 | **system (user / role / device / invitation)** | `SysUser`, `SysUserArchive`, `SysUserDevice` (login-anomaly detection), `SysUserInvitation`, `SysActionLog`, role listing, archive rename/delete_slot. | Medium | **Done** — device registration + access-policy checks are wired |
| 6 | **BinaryMD5 archive export** | The GZIP-compressed, BinaryMD5-keyed producer for `item_doc` / `marker_doc` / `marker_link_doc` / `icon_doc` / `tag_doc`, with an in-process moka cache (3600s TTL) plus a Redis second level, and admin refresh endpoints. | High | **Done** |
| 7 | **OAuth2 / JWKS** | `/oauth/token` (password / QQ / refresh_token / client_credentials), `/.well-known/jwks.json`, access-policy checks, scope mapping. | High | **Done** — RS256 signing with RSA JWKS; **JWK rotation** via `JWT_RSA_VERIFY_KEYS` (historical public keys stay verifiable + published; see the rotation steps below). In HS256 mode the JWKS endpoint returns an empty key set (the HMAC secret is never disclosed) |

## Current state

- All seven batches are landed end-to-end (entity → DTO → business → route →

test). Business assertions live in `tests/rust/tests/api_db_test.rs`
(live DB, CI `DB integration` job): area CRUD, `item_doc` BinaryMD5, marker
tweak, OAuth policy/device/QQ login, JWKS, cache stability and refresh.
(The punctuate audit is deprecated — its routes and models were removed;
the `marker_punctuate` staging table remains part of the schema.)

- The `SafeEntityTrait` + `impl_safe_operation!` macros are stable; new

domains reuse the template.

## Known gaps (remaining differences vs Java)

- In HS256 mode JWKS serves an empty key set (the HMAC secret is never

published); JWK rotation is RS256-only (see batch 7, covered by
`tests/rust/tests/jwks_rotation_test.rs`).

- Database schema drift against the production database still needs data

validation (`marker_linkage` nullable columns, `sys_user_archive`
structure binding, …).

- Docs are maintained in en / zh-Hans / zh-Hant only, kept in sync.

## 2026-10 parity wave (frozen Java + latest map_register_v3)

Endpoint-by-endpoint re-audit against Java `dev-single` (frozen 2026-09)
and `map_register_v3` `origin/dev` (2026-09-23, alova + static protobuf
decoders) closed these gaps:

- **`GET /api/marker_doc/list_markers`** — full-marker export (GZIP'd

`MarkerVoList` protobuf with a creator/updater users map that includes
soft-deleted users, per Java `selectUserWithDelete`). Consumed by the
4.19+ frontend via its static protobuf decoder.

- **`GET /api/res/get`** — real implementation of the dedup-upload probe

(`?filePath=` → MinIO stat → `{filePath, fileUrl}` or an empty object).

- **`PUT /api/res/upload/image`** — the object key is now the client

`filePath` (Java contract: same path reused, duplicates rejected with
the Java error text), plus path-safety validation.

- **`DELETE /api/cache/icon`** — Java `CacheController` contract

(`List<Long>` body, empty list = purge all); the legacy `iconTag` /
`icon_tag` routes remain.

- **Icon VO realignment** — `name` → `tag` plus the full Java field set

(`creatorId/typeIdList/urlVariants/description/…`); search accepts
`tag` (with a `name` alias).

- **`POST /api/marker/get/list_byid`** accepts both body shapes (bare ID

array and the `{"markerIdList":[…]}` object).

- **Envelope fixes** — `users` maps now carry the full

`SysUserSmallVo` fields; item/icon lists attach `users`; every
paginated list emits `PageListVo.size`.

- **`POST|GET /oauth/token`** — GET method and

`application/x-www-form-urlencoded` bodies accepted (Java Spring
OAuth2 full compatibility); multipart login and query refresh unchanged.

- **`/system/invitation/info`** returns the `{code, username}` small VO

instead of leaking the raw entity.

- **`HistoryOperationType`** gains `IconTag = 5`; marker-create request

fields relaxed to serde defaults (Java null tolerance).

## 2026-10 second wave (leftover items landed)

- **History write path**: marker update/delete/tweak and item update/delete

now record history rows (pre-change camelCase DTO JSON with md5); the
Rust side previously never wrote history, leaving both the history list
and score generation without a data source.

- **Score generation aligned with the Java algorithm**: consecutive

history-pair diffs (fields per changed field, chars via Levenshtein
distance over `markerTitle`/content), synthesized boundary records
(created markers start from their first linked item; the final value is
the first post-span history or the marker's current state), credited to
the before-record's creator × Asia/Shanghai day bucket.

- **Image URL rewriting unified**: `ICON_PROXY_BASE` now also applies to

icon responses and res `fileUrl` (ddns.minemc.top has no CORS headers).

- **`item_type` update request** hiddenFlag relaxed to a serde default.
- **WS contract update**: the latest frontend confirmed the socket.io

migration (see the 2026-10-22 update in ws-frontend-contract); the
pre-migration local tree's `?token=` patch lives on the frontend repo's
`fix/ws-token-query` branch.

- **WS migration decision landed**: the backend serves a Socket.IO-compatible
  channel via `socketioxide` (`/socket.io`); see the ws-frontend-contract
  2026-10-23 decision for the auth rules.

## Follow-ups

- Remaining gap items live in the iteration backlog (root `PLAN.md`) and

land through the master-based PR flow.
