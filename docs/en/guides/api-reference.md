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
authenticated session (except the public invitation and QQ-registration
endpoints), with the administrative verbs gated one level further by
`ExtractAdmin`. The default body limit is 16 MiB. Server-pushed events
(announcements, cache purges) are broadcast over the `/ws/{userId}`
WebSocket endpoint.

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
| Tag | `/api/tag` | Free-form labels applied to markers for grouping. |
| Tag type | `/api/tag_type` | Classification taxonomy for tags. |

## Compressed archive exports — BinaryMD5 bundles

The `*_doc` domains serve GZIP-compressed archives the client downloads to
bootstrap offline. The archives are GZIP-compressed JSON blobs keyed by the
MD5 of the compressed bytes, generated from PostgreSQL on a cache miss and
then held in an in-process moka cache with a Redis second level shared
across replicas.

| Domain | Prefix | Purpose |
| --- | --- | --- |
| Icon doc | `/api/icon_doc` | Single-blob GZIP-compressed icon-archive download (`all_bin`). |
| Item doc | `/api/item_doc` | Paginated GZIP-compressed item-archive download (`list_page_bin`). |
| Marker doc | `/api/marker_doc` | Paginated GZIP-compressed marker-archive download. |
| Marker link doc | `/api/marker_link_doc` | Single-blob marker-link archives: flat list and adjacency graph (`all_list_bin` / `all_graph_bin`). |
| Tag doc | `/api/tag_doc` | Single-blob GZIP-compressed tag-archive download (`all_bin`). |

## Cache invalidation — admin refresh endpoints

The `cache` domain exposes admin-only `DELETE` endpoints (mirroring the Java
`CacheService.clean*` surface) that flush server-side caches so clients
refetch fresh data on their next poll. Each sub-route mirrors a content
domain; today the item / marker / marker-link handlers flush the BinaryMD5
caches (and broadcast a WebSocket purge event), while the remaining
sub-routes are no-ops pending their cache layers. `POST /app/trigger/update`
flushes every BinaryMD5 cache in one shot.

| Domain | Prefix | Purpose |
| --- | --- | --- |
| Cache | `/api/cache/{area,common_item,icon_tag,item,marker,marker_link,notice}` | Admin cache-refresh (DELETE) endpoints per content type. |

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
| OAuth | `/oauth/token` | OAuth2 token issuance (JWT + JWKS, RS256 or HS256; grants: password, QQ, refresh_token, client_credentials). |

## Notes on parity

This router is the Rust port of the Java controllers, not a clean-room
redesign. When a route exists on the Java side but not yet here, it is
tracked in the [Java Sync Roadmap](./sync-with-java-roadmap.md). New routes
must follow the five-layer pattern documented in the
[Domain Sync Template](./domain-sync-template.md) so the entity, DTO, business
logic, route, and smoke-test layers all land together.
