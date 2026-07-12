# Java Sync Roadmap

This roadmap tracks the port of features from the Java reference backend
([`java-genshin-map-cloud`](https://github.com/kongying-tavern/java-genshin-map-cloud))
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
Two-tier cache on the Java side (Caffeine → port to `moka` / `quick-cache`).

- **Crowd-sourced punctuate workflow**: user marker submissions → staging

table → audit → promotion to live markers (three-state approval).

- **System**: user, role, device (login-anomaly detection), invitation,

`action_log`, archive.

- **Auth**: OAuth2 password-grant JWT with a JWKS endpoint, RSA keypair,

device/IP anomaly detection on token issuance.

## Porting priority

The order below front-loads the entities that unblock the most downstream
work. Each step lists the key entity/feature and an estimated complexity.

| # | Area | Key entity / feature | Complexity | Status |
| --- | --- | --- | --- | --- |
| 1 | **area + marker** (reference samples) | `Area`, `Marker`, `hiddenFlag`, `specialFlag`. Establishes the `SafeEntityTrait` pattern and the five-layer domain template every later port copies. | Medium | Done — used as the porting template |
| 2 | **icon / item / tag families** | `Icon`, `IconType`, `IconTypeLink`, `Item`, `ItemType`, `ItemTypeLink`, plus the `Tag` / `TagType` taxonomy. Includes the `selectPageItemByCondition` `specialFlag` filter and the icon-tag merge. | Low–Medium | In progress |
| 3 | **notice / route / history** | `Notice` (validity-sort rule), `Route`, `History`. Read-heavy content that pairs naturally with the Redis cache layer. | Low–Medium | In progress |
| 4 | **punctuate workflow + scoring** | `MarkerPunctuate` staging → `Marker` promotion (three-state audit) and `ScoreStat` aggregation (scope/span bucketing). Two-phase state machine plus the score aggregate. | High | Planned |
| 5 | **system (user / role / device / invitation)** | `SysUser`, `SysUserArchive`, `SysUserDevice` (login-anomaly detection), `SysUserInvitation`, `SysActionLog`, role listing. Depends on the bcrypt hashing already in `utils`. | Medium | In progress |
| 6 | **BinaryMD5 archive export** | The GZIP-compressed, BinaryMD5-keyed producer for `item_doc` / `marker_doc` / `marker_link_doc`. The Rust side generates GZIP-compressed blobs on-the-fly from PostgreSQL per request (no caching yet); porting the two-tier cache layer (Redis or moka) is the work. | High | Planned |
| 7 | **OAuth2 / JWKS** | `/oauth/token` issuance, JWKS publication, the RSA keypair + token enhancer, device/IP anomaly check, and the `qq` registration provider. Depends on `jsonwebtoken` 10 (already pinned). | High | Planned |

## Notes

- Steps 1–5 are CRUD-shaped ports that follow the

[Domain Sync Template](./domain-sync-template.md); they are low-risk and
can land incrementally.

- Steps 4, 6, and 7 each carry significant business or algorithmic logic

(the punctuate state machine, BinaryMD5 hashing + GZIP streaming, JWKS key
rotation). Each should get its own design note under `docs/en/designs/`
before implementation.

- Update this table's Status column (and `CHANGELOG.md`) as each domain moves

from *Planned* → *In progress* → *Done*.
