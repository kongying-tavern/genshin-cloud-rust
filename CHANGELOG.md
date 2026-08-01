# Changelog

All notable changes to the 空荧酒馆·原神地图 Rust backend (Genshin Map Cloud Rust)
will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Infrastructure (master-based iteration transition)

- Switch to the master-based PR workflow: the `dev` branch was
  squash-merged into master ([#18](https://github.com/langyo/genshin-cloud-rust/pull/18))
  and archived as tag `archive/dev-snapshot`; every new patch now lands
  via its own PR against master. Branch protection is enabled on master
  (require PR, 6 required status checks, linear history, no force-push).

- Fix three latent CI bugs: the manual sccache install referenced the
  wrong extracted directory name (replaced with
  `mozilla-actions/sccache-action@v0.0.11` on both OSes); the global
  `RUSTC_WRAPPER=sccache` broke Windows jobs where sccache was absent;
  Trufflehog rejected the duplicated `--fail` flag in `extra_args`.

- Harden CI ([#19](https://github.com/langyo/genshin-cloud-rust/pull/19)):
  the commit-msg lint now uses the org reusable workflow (lints the PR
  title and every commit); added the cargo-deny workflow (advisories,
  bans, licenses, sources); allowed the four permissive licenses required
  by the dependency graph (`bzip2-1.0.6`, `NCSA`, `CDLA-Permissive-2.0`,
  `BSL-1.0`) and ignored RUSTSEC-2023-0071 with justification (the `rsa`
  crate is a transitive-only dependency that is never exercised — the
  workspace uses HMAC JWT exclusively). Dropped the retired `dev` branch
  from all workflow triggers.

- Add [PLAN.md](./PLAN.md): the iteration plan — unfinished-work
  inventory, the master-based PR ruleset, and the milestone backlog
  (M1 infra & test harness, M2 tech-debt cleanup, M3 authZ/OAuth,
  M4 caching, M5 docs & release).

- Rewrite the Dockerfile as a working multi-stage build: the previous
  image could not build (duplicate `cargo new _utils`, missing
  `COPY --from`, `ENTRYPOINT ["./a"]` pointing at a non-existent
  binary) and shipped unrelated `wasm32`/`cargo-make` leftovers. The
  new image builds `_router` with LTO in a `rust:1` builder (BuildKit
  cache mounts) and runs it on `debian:bookworm-slim` with
  `ca-certificates` + `tini`. A `Docker` CI workflow validates the
  build on every PR, and `.dockerignore` keeps the context lean.

- Add the DB-backed integration test harness: a `user_db` test binary
  that provisions the `sys_user` table on a live Postgres via sea-orm's
  `Schema` and exercises the `SafeEntityTrait` round-trip (insert →
  read → soft-delete). Gated on `GCS_TEST_DB` so `cargo test --workspace`
  stays green without a database; the new `integration` CI job
  provisions Postgres and runs it. The `tests/docker` compose comment
  no longer references the removed `#[ignore]` convention.

- Add the `api_db` business-assertion test: seeds area + item rows on a
  live Postgres (tables built FK-free via DDL rewriting so each domain
  is independent) and asserts the business-layer functions return real
  data — `area::do_list`/`do_add` and the BinaryMD5 `item_doc`
  pipeline. Upgrades the e2e smoke checks (which treated 401/403 as
  "route exists ✓") into actual data assertions. Runs in the same
  `integration` CI job.

- Wire the archive rename handler: the route previously stubbed the
  response (it could not call `do_rename` because `auth` was moved into
  `do_get_last`). New `do_rename_by_slot(user_id, slot_index, name)`
  renames the latest archive in the slot; the route now returns the real
  operation result.

- Implement the archive `delete_slot` handler: new
  `do_delete_slot(user_id, slot_index)` soft-deletes every archive in
  the slot; the route previously returned a stub `{}`.

- Define `RouteVO` and return real route data: `do_get_page` /
  `do_get_search` / `do_get_list_by_id` previously mapped their (correct)
  queries into the `RouteEmptyResponse` placeholder. They now return
  `RoutePageResponse { total, items }` / `Vec<RouteVO>` with full route
  fields (marker list, hidden flag, extra, creator info, timestamps).

- Remove the user-domain placeholder implementations (PLAN.md F8):
  `do_register` / `do_register_qq` now take an explicit initial password
  (no more hard-coded `"default_password"`); `do_update_password`
  verifies the old password before storing the new one; `do_list`
  applies the whitelisted sort keys (`createTime+/-`, `id+/-`,
  `nickname+/-`) instead of ignoring them; `do_kick_out` clears the
  user's Redis sessions (degrades to no-op without Redis). The
  `user_db` test now asserts all of the above plus `do_delete`.

- Implement the marker `ItemList` tweak (PLAN.md F9): the
  `do_tweak` handler previously skipped the `itemList` prop entirely.
  It now maintains the `marker_item_link` table for
  Append / InsertIfAbsent / InsertOrUpdate / Merge / Update (upsert
  with count), Replace (rebuild), and RemoveLeft / RemoveRight
  (remove listed links). `api_db` test covers the full cycle against
  live Postgres.

- Enforce roles and atomicity in the punctuate audit (PLAN.md F6):
  `do_pass` / `do_reject` / `do_delete` now require the Admin or
  MapManager role (other roles get an explicit error). `do_pass`
  executes the "write marker + delete punctuate" steps in a single
  database transaction, so a failure rolls back both. `api_db` test
  asserts the role gate, reject with remark, and the atomic pass
  promotion against live Postgres.

- Enforce OAuth access policies and scope mapping (PLAN.md F7):
  password login now checks the user's `access_policy` against
  `sys_user_device` history — `ip:same_last_ip` / `dev:same_last_device`
  reject mismatched environments, `ip:block_disallow_ip` /
  `dev:block_disallow_device` reject entries marked disabled
  (`status != 0`); successful logins register/refresh the device row.
  `oauth_client_credentials` now validates the scope string (`all` /
  empty → `All`, anything else rejected) instead of silently returning
  `All`. `api_db` test covers the policy gates, device registration,
  and scope mapping against live Postgres.

- Add the JWKS endpoint (PLAN.md M3): `GET /.well-known/jwks.json`
  publishes the current HMAC (HS256) key in RFC 7517 `oct` form
  (`kty: oct`, base64url `k`), unauthenticated. New
  `jwt_secret_raw()` accessor keeps the key material single-sourced.
  `api_db` test verifies the key material round-trips with the JWT
  secret.

- Implement QQ third-party login (PLAN.md M3): `POST /oauth/qq`
  exchanges a QQ openid for a local token, matching `sys_user.qq`
  (bound via `/user/register/qq`, which now stores the openid).
  `oauth_qq_login` runs the same access-policy checks, device
  registration, and login logging as password login; token issuance
  was extracted into a shared `issue_token` helper. Unregistered
  openids fail with an explicit error. `api_db` test covers both the
  bound and unbound paths against live Postgres.

- Fix the systemic identity-column bug: every business insert
  hard-coded `id: Set(0)` on `GENERATED BY DEFAULT AS IDENTITY`
  primary keys. The first insert into an empty table succeeded
  (id=0), but **any second insert collided on the primary key**
  (`duplicate key value violates unique constraint`). All ~20
  call sites across area/archive/icon/icon_type/item/item_common/
  item_type/marker/marker_link/notice/punctuate/punctuate_audit/
  route/tag/tag_type now use `NotSet` so the identity column
  assigns ids. The `api_db` test adds a repeated `area::do_add`
  regression assertion.

- Add the in-process BinaryMD5 page cache (PLAN.md M4/F5): item,
  marker, and marker-link doc endpoints now serve their GZIP pages
  from a moka cache (Java's Caffeine equivalent, 300s TTL) instead of
  re-serializing the whole dataset on every request. The md5-list
  `time` field is now the page's generation timestamp (stable within
  the cache TTL) rather than the request time, so clients no longer
  see spurious "data changed" signals. `binary_doc` exposes
  `get_or_compute` / `invalidate` for the future refresh wiring.
  `api_db` test asserts the md5 + time are stable across repeated
  calls.

- Wire the cache-refresh endpoints (PLAN.md M4/F10): `DELETE
  /cache/item`, `/cache/marker`, and `/cache/marker_link` now flush
  the in-process BinaryMD5 cache (`binary_doc::invalidate_all`)
  instead of being no-ops. The remaining domains (area / common_item /
  icon_tag / notice) have no in-process cache yet and stay honest
  no-ops. `api_db` test asserts the refresh regenerates pages (fresh
  `time`).

- Resync the zhs/en Java-sync roadmaps with the actual domain status
  (PLAN.md M5/D1): both versions now carry the same Status column —
  batches 1–3, 5, 6 done; batches 4 and 7 mostly done with the
  remaining gaps (score field-level diff, RSA/JWKS rotation) stated
  explicitly. The stale "follow-up" items (sea-orm 2.x / minio 0.4
  migrations, which were completed long ago) are removed.

- Make the CDN proxy configurable (PLAN.md M5/I4): the `/cdn` upstream
  was hard-coded to `v3.yuanshen.site` and the dadian config was a
  fake empty blob. New `CDN_UPSTREAM` env var overrides the upstream
  (self-hosted CDN / internal mirror); `CDN_DADIAN_CONFIG` points at a
  pre-generated bz2 config served for `/cdn/dadian-preview.json.bz2`
  (falling back to the empty dev config when unset). Both are
  documented in `.env.example`.

### Dependencies (dev branch)

- Upgrade the workspace to edition 2024 across all four packages

(`_utils`, `_database`, `_functions`, `_router`); `rust-toolchain.toml`
pins stable with rustfmt + clippy.

- Bump cross-major dependencies to their latest stable lines: `reqwest`

^0.12 → ^0.13, `redis` ^0.32 → ^1, `axum-extra` ^0.10 → ^0.12,
`tower-http` ^0.6 → ^0.7, `bcrypt` ^0.17 → ^0.19, `jsonwebtoken` ^9 → ^10,
`md5` ^0.7 → ^0.8, `oneshot` ^0.1 → ^0.2, `flume` ^0.11 → ^0.12,
`strum` ^0.26 → ^0.28.

- **Strip all `aws-*` crates from the dependency graph.** The workspace now

pins `rustls` with `default-features = false` and only the `ring` provider;
`reqwest` uses `rustls-no-provider`. Verified: no `aws-` package remains in
`cargo tree`.

- **sea-orm** upgraded to `^2.0.0-rc`. The `SafeEntityTrait` macro and all 33

business call sites have been ported to the new `ValidatedUpdateOne` API
(`.validate().map(...)` pattern in the macro, `?` before `.exec()` at call
sites). `strum` bumped to ^0.28 to match.

- **minio** upgraded to `^0.4`. `Client` → `MinioClientBuilder`, bucket

provisioning uses `.bucket_exists()?.build().send()` + `S3Api` trait.

### Known technical debt (dev branch)

- `cargo clippy --workspace --all-targets -- -D warnings` passes with zero

errors. CI enforces strict clippy.

- Archive `rename` handler: `auth` is moved by `do_get_last`, preventing

`do_rename` from being called (TODO in code). Business functions should be
refactored to borrow `&AuthInfo`.

- Archive `delete_slot`: needs a dedicated `do_delete_slot(user_id, slot_index)`

function (TODO in code).

- Route `do_get_page` / `do_get_search` / `do_get_list_by_id`: queries are

correct but results map to `RouteEmptyResponse` placeholder until a
`RouteVO` type is defined (TODO in code).

- BinaryMD5 `*_doc` endpoints: no in-process cache (Java uses Caffeine);

each request regenerates. A Redis or moka cache layer should be added.

- Score `do_generate_score`: simplified aggregation (counts edits per

contributor). Java's full field-level diff algorithm (`ScoreDataPunctuateVo`)
is not yet ported.

### Tooling

- Install the `celestia-devtools` commit-msg hook enforcing the org gitmoji

convention (English subject, capitalized, trailing period).

- Replace the merge commit on `master` with a single squashed commit to keep

the history linear and compliant with the hook's master-merge-guard.

- Add a `justfile` (verb-first dispatch) that imports the vendored

`celestia-devtools.just` recipes.

- Add `rust-toolchain.toml` (stable + rustfmt + clippy), `rustfmt.toml`, and

`.editorconfig` for consistent formatting across contributors.

- Add `.cargo/config.toml` with `git-fetch-with-cli` and the Windows 8 MiB

stack bump; machine-specific `[patch]` overrides stay in user-level config.

- Add `.gitattributes` to normalize line endings to LF.
- Modernize CI: replace the deprecated `actions-rs` workflow with

`dtolnay/rust-toolchain`-based `rust.yml`, add a multi-OS `test.yml` with a
secrets scan, a `docs.yml` for multilingual docs, and `dependabot.yml`.

- Add GitHub community files: `PULL_REQUEST_TEMPLATE.md`, `SECURITY.md`,

`CODE_OF_CONDUCT.md`, and issue templates.

- Add `deny.toml` (cargo-deny policy) for license and advisory gating.

### Documentation

- Rewrite `ReadMe.md` → `README.md` in the celestia-island multilingual format

(centered header, badge row, language switcher, quick start, architecture,
documentation index).

- Lay the groundwork for multilingual docs under `docs/` (English and

Simplified Chinese first; remaining languages scaffolded).

### Notes

- The commit messages on `master` prior to the hook are a mix of Chinese and

gitmoji; from the hook-install commit forward, all new commits follow the
org gitmoji convention (English subject line).

- The `noa` co-author hook is reserved and not installed yet — it requires a

built `noa` binary and the entelecheia chat-log/aporia configuration, neither
of which is present in this repo's environment.
