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

## Known gaps (remaining parity with Java)

- Database schema deviations from the real database await data validation
  (`marker_linkage` nullable columns, `sys_user_archive` structure binding).
- Translation: the docs ship in three maintained languages — English
  (`docs/en`), Simplified Chinese (`docs/zh-Hans`), and Traditional Chinese
  (`docs/zh-Hant`).

## Rotating the JWT signing key (RS256)

The signing key is `JWT_RSA_PRIVATE_KEY_PEM`; `JWT_RSA_VERIFY_KEYS`
(comma-separated RSA **public** key PEMs) lists historical keys that must
still verify tokens. The JWKS endpoint publishes the current key (`kid`
`genshin-cloud-rsa-v1`) first, then each historical key in order (`v2`,
`v3`, ...). Rotation is a two-step deploy that never invalidates live
tokens:

1. Generate the new keypair, add the **new** public key to
   `JWT_RSA_VERIFY_KEYS` (keeping the old one), and deploy. Tokens are
   still signed with the old key; both verify.
2. Switch `JWT_RSA_PRIVATE_KEY_PEM` to the new private key and deploy.
   Tokens are now signed with the new key (kid stays `v1`); the old key
   remains in `JWT_RSA_VERIFY_KEYS` for verification.
3. On the next rotation, drop the oldest public key from
   `JWT_RSA_VERIFY_KEYS`.

HS256 (`JWT_SECRET`) has no rotation story beyond a restart — the JWKS
never publishes the HMAC secret.

## Follow-up

- The gaps above are tracked in the iteration backlog (`PLAN.md` at the repo
  root) and land via the master-based PR workflow.
