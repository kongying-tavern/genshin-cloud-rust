# API Reference

This is an overview of the HTTP surface the router exposes, grouped by
purpose. Every domain below is a direct Rust port of the corresponding Java
controller in `genshin-map-cloud`; the request/response shapes match so
the existing front-end works against either backend. For exact route paths
and payload fields, read the relevant module under
`packages/router/src/routes/`.

All `/api/*` routes run behind the `ExtractAuthInfo` middleware, which parses
the `Authorization: Bearer <jwt>` header into an `AuthInfo` threaded through
every business function. The `/system/*` routes additionally require an
authenticated session. The default body limit is 16 MiB.

## Map content — the map data the front-end renders

| Domain | Prefix | Purpose |
| --- | --- | --- |
| Area | `/api/area` | Region tree (continents → countries → sub-areas). CRUD plus parent/child traversal. |
| Icon | `/api/icon` | Marker icon assets and their metadata; uploaded images live in MinIO. |
| Icon type | `/api/icon_type` | Classification taxonomy for icons (groups, categories). |
| Item | `/api/item` | In-game items attachable to markers; includes copy and join operations. |
| Item type | `/api/item_type` | Classification taxonomy for items. |
| Item common | `/api/item_common` | Shared/cross-region item definitions reused across areas. |
| Marker | `/api/marker` | The core entity — a single point of interest on the map. |
| Marker link | `/api/marker_link` | Linkages between markers (e.g. "this cave entrance connects to that exit"). |

## Compressed archive exports — BinaryMD5 bundles

The `*_doc` domains serve GZIP-compressed archives the client downloads to
bootstrap offline. The archives are GZIP-compressed JSON blobs keyed by the
MD5 of the compressed bytes, generated on-the-fly from PostgreSQL by the
Rust backend. A process-internal cache is planned to avoid regenerating on
every request.

| Domain | Prefix | Purpose |
| --- | --- | --- |
| Item doc | `/api/item_doc` | Paginated GZIP-compressed item-archive download (`list_page_bin`). |
| Marker doc | `/api/marker_doc` | Paginated GZIP-compressed marker-archive download. |
| Marker link doc | `/api/marker_link_doc` | Paginated GZIP-compressed marker-link-archive download. |

## Read-through cache — fast front-end bootstrap

The `cache` domain serves precomputed snapshots straight from Redis so the
map client can cold-start without hitting PostgreSQL. Each sub-route mirrors
a content domain.

| Domain | Prefix | Purpose |
| --- | --- | --- |
| Cache | `/api/cache/{area,common_item,icon_tag,item,marker,marker_link,notice}` | Hot, Redis-backed read views per content type. |

## Community workflow — scoring

| Domain | Prefix | Purpose |
| --- | --- | --- |
| Score | `/api/score` | Per-user contribution scoring (score_stat aggregate). |
| Notice | `/api/notice` | In-app announcements and broadcast messages. |
| History | `/api/history` | Audit history of edits to content entities. |
| Resources | `/api/res` | Static resource references (versioned asset pointers). |

## System — accounts, devices, invitations, OAuth

Mounted under `/system/*`, these are the administration and account surface.

| Domain | Prefix | Purpose |
| --- | --- | --- |
| User | `/system/user` | Registration (incl. QQ), info, update, password change, kick-out, list. |
| Role | `/system/role` | Role listing for RBAC checks. |
| Device | `/system/device` | Trusted-device management. |
| Invitation | `/system/invitation` | Invitation-code generation, consumption, listing. |
| Archive | `/system/archive` | Per-user save-slot archives (get/put/save/rename/restore/delete). |
| Action log | `/system/action_log` | Audit log of administrative actions. |
| OAuth | `/oauth/token` | OAuth2 token issuance (JWT + JWKS, port in progress). |

## Notes on parity

This router is the Rust port of the Java controllers, not a clean-room
redesign. When a route exists on the Java side but not yet here, it is
tracked in the [Java Sync Roadmap](./sync-with-java-roadmap.md). New routes
must follow the five-layer pattern documented in the
[Domain Sync Template](./domain-sync-template.md) so the entity, DTO, business
logic, route, and smoke-test layers all land together.
