# Changelog

All notable changes to the 空荧酒馆·原神地图 Rust backend (Genshin Map Cloud Rust)
will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Fixed

- Port the score field-level weighting (the last known Java-parity gap from
  the 0.2.0 release): `do_generate_score` now filters history to the
  `Position` (type=4) rows and weights each contribution by the number of
  fields in its content JSON (Added/Modified by field count, Deleted = 1),
  instead of counting every edit as 1. `do_get_score_data` reads the real
  score from `score_stat.content` (`fieldWeight`, falling back to `count`)
  instead of returning a fixed 1.0 per row. `api_db` test asserts the
  weighting (3-field + 1-field rows → score 4, non-Position rows ignored)
  and the read-back.

- Resync the gap documentation: the roadmap batch-4 status is now **Done**
  (score weighting landed), and the punctuate-workflow design doc drops the
  stale "audit permission" / "transactionality" / "field diff" follow-ups
  (all fixed in the M2/M3 PRs); only the RSA/JWKS rotation gap remains.

### Security

- Add RS256 signing with RSA JWKS (the last roadmap gap): when
  `JWT_RSA_PRIVATE_KEY_PEM` is set (PKCS#8 or PKCS#1 PEM), tokens are
  signed with RS256 and `/.well-known/jwks.json` publishes the RSA
  public key (`kty: RSA`, base64url `n`/`e`). Without it, the
  workspace keeps HS256 (`kty: oct`) as before. Verification accepts
  both algorithms (active first, then fallback), so tokens signed
  before an RSA migration stay valid. `api_db` test generates an
  ephemeral RSA key and exercises the whole flow (login → RS256
  sign/verify → JWKS RSA shape) end-to-end. `.env.example` documents
  the knob; the deny.toml RUSTSEC-2023-0071 justification is updated
  (the `rsa` crate now performs signing/key export, never RSA
  decryption).

### Documentation

- Translate the nine scaffolded doc entry pages (ar/de/es/fr/ja/ko/pt/ru/zht):
  each language's `README.md` is now a real translated landing page (project
  intro + full document index linking to the English content, which remains
  the source of truth), and every `SUMMARY.md` language switcher now lists
  all 11 languages with the current one highlighted. The previously
  Chinese-only placeholders and truncated switchers are gone.

- Clarify the stale schema TODOs (PLAN.md F11, docs-only): the
  `marker_linkage` "cannot be null" TODOs contradicted the already
  non-null field types (removed); its `path` FIXME now states why the
  type stays loose (no real data samples yet). `sys_user_archive.data`
  documents the intentional opaque-JSON design, and `sys_user_device.status`
  documents the `0 = normal / non-zero = disabled` convention used by the
  OAuth access-policy checks.

### Fixed (legacy-database alignment)

- Align `area` / `item` / `item_type` with the legacy Java database: the
  `icon_id` (bigint) column is replaced with the old schema's
  `icon_tag` (varchar, e.g. `"C:FD"`), and the fake `parse::<i64>()`
  resolution of the tag string is removed — the tag is now stored and
  returned verbatim. The bogus icon foreign-key relations are dropped
  (the old schema has no such FK). Verified against the full 461 MB
  legacy dump: the remaining 22 shared tables now match column-for-column.

- Align the `icon` entity with the legacy Java database: the old schema
  has only `name` + `url` columns; the `tag` / `description` /
  `url_variants` columns (and the now-unused `IconURLVariants` type)
  are removed, the entity and `IconVO` now expose `name` (the request
  model already used `name` — `do_add` previously mis-stored it into
  `tag`). The name filter now queries `name` instead of `tag`.

- Drop the `tag` / `tag_type` columns that the legacy database does not
  have (`hidden_flag` / `sort_index`): the entities, VOs, and business
  code are aligned so the schema now matches the old DDL exactly.

## [0.2.0] - 2026-08-01

### Infrastructure (master-based iteration transition)

- Switch to the master-based PR workflow: the `dev` branch was
  squash-merged into master ([#18](https://github.com/langyo/genshin-cloud-rust/pull/18))
  and archived as tag `archive/dev-snapshot`; every new patch now lands
  via its own PR against master. Branch protection is enabled on master
  (require PR, 8 required status checks, linear history, no force-push).

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
  inventory, the master-based PR ruleset, and the milestone backlog.

- Rewrite the Dockerfile as a working multi-stage build: the previous
  image could not build (duplicate `cargo new _utils`, missing
  `COPY --from`, `ENTRYPOINT ["./a"]` pointing at a non-existent
  binary) and shipped unrelated `wasm32`/`cargo-make` leftovers. The
  new image builds `_router` with LTO in a `rust:1` builder (BuildKit
  cache mounts) and runs it on `debian:bookworm-slim` with
  `ca-certificates` + `tini`. A `Docker` CI workflow validates the
  build on every PR, and `.dockerignore` keeps the context lean.

### Testing

- Add the DB-backed integration test harness: a `user_db` test binary
  that provisions the `sys_user` table on a live Postgres via sea-orm's
  `Schema` and exercises the `SafeEntityTrait` round-trip (insert →
  read → soft-delete). Gated on `GCS_TEST_DB` so `cargo test --workspace`
  stays green without a database; the new `integration` CI job
  provisions Postgres and runs it. The `tests/docker` compose comment
  no longer references the removed `#[ignore]` convention.

- Add the `api_db` business-assertion test: seeds rows on a live
  Postgres (tables built FK-free via DDL rewriting so each domain is
  independent) and asserts the business-layer functions return real
  data — area CRUD, the BinaryMD5 `item_doc` pipeline, marker tweak,
  punctuate audit (role gate + transactional promotion), OAuth
  policy/device/QQ login, JWKS, and cache stability/refresh. Upgrades
  the e2e smoke checks (which treated 401/403 as "route exists ✓")
  into actual data assertions.

### Fixes

- Wire the archive rename handler: the route previously stubbed the
  response. New `do_rename_by_slot(user_id, slot_index, name)` renames
  the latest archive in the slot.

- Implement the archive `delete_slot` handler: new
  `do_delete_slot(user_id, slot_index)` soft-deletes every archive in
  the slot.

- Define `RouteVO` and return real route data: `do_get_page` /
  `do_get_search` / `do_get_list_by_id` previously mapped their (correct)
  queries into the `RouteEmptyResponse` placeholder. They now return
  `RoutePageResponse { total, items }` / `Vec<RouteVO>` with full route
  fields.

- Remove the user-domain placeholder implementations:
  `do_register` / `do_register_qq` now take an explicit initial password
  (no more hard-coded `"default_password"`); `do_update_password`
  verifies the old password before storing the new one; `do_list`
  applies the whitelisted sort keys (`createTime+/-`, `id+/-`,
  `nickname+/-`); `do_kick_out` clears the user's Redis sessions
  (degrades to no-op without Redis).

- Implement the marker `ItemList` tweak: the `do_tweak` handler
  previously skipped the `itemList` prop entirely. It now maintains
  the `marker_item_link` table for Append / InsertIfAbsent /
  InsertOrUpdate / Merge / Update (upsert with count), Replace
  (rebuild), and RemoveLeft / RemoveRight.

- Fix the systemic identity-column bug: every business insert
  hard-coded `id: Set(0)` on `GENERATED BY DEFAULT AS IDENTITY`
  primary keys — any second insert collided on the primary key. All
  call sites now use `NotSet` so the identity column assigns ids.

### Security

- Enforce roles and atomicity in the punctuate audit: `do_pass` /
  `do_reject` / `do_delete` now require the Admin or MapManager role;
  `do_pass` executes the "write marker + delete punctuate" steps in a
  single database transaction.

- Enforce OAuth access policies and scope mapping: password login now
  checks the user's `access_policy` against `sys_user_device` history
  (`ip:same_last_ip` / `dev:same_last_device` reject mismatched
  environments, `ip:block_disallow_ip` / `dev:block_disallow_device`
  reject disabled entries); successful logins register/refresh the
  device row. `oauth_client_credentials` validates the scope string
  instead of silently returning `All`.

- Add the JWKS endpoint: `GET /.well-known/jwks.json` publishes the
  current HMAC (HS256) key in RFC 7517 `oct` form, unauthenticated.

- Implement QQ third-party login: `POST /oauth/qq` exchanges a QQ
  openid for a local token, matching `sys_user.qq` (bound via
  `/user/register/qq`); token issuance is shared via `issue_token`.

### Performance

- Add the in-process BinaryMD5 page cache: item, marker, and
  marker-link doc endpoints now serve their GZIP pages from a moka
  cache (Java's Caffeine equivalent, 300s TTL) instead of
  re-serializing the whole dataset on every request. The md5-list
  `time` field is now the page's generation timestamp (stable within
  the cache TTL).

- Wire the cache-refresh endpoints: `DELETE /cache/item`,
  `/cache/marker`, and `/cache/marker_link` now flush the in-process
  BinaryMD5 cache instead of being no-ops.

### Configuration

- Make the CDN proxy configurable: the `/cdn` upstream was hard-coded
  to `v3.yuanshen.site` and the dadian config was a fake empty blob.
  New `CDN_UPSTREAM` env var overrides the upstream; `CDN_DADIAN_CONFIG`
  points at a pre-generated bz2 config for `/cdn/dadian-preview.json.bz2`
  (falling back to the empty dev config when unset). Documented in
  `.env.example`.

### Dependencies

- Upgrade the workspace to edition 2024 across all four packages
  (`_utils`, `_database`, `_functions`, `_router`); `rust-toolchain.toml`
  pins stable with rustfmt + clippy.

- Bump cross-major dependencies to their latest stable lines: `reqwest`
  ^0.12 → ^0.13, `redis` ^0.32 → ^1, `axum-extra` ^0.10 → ^0.12,
  `tower-http` ^0.6 → ^0.7, `bcrypt` ^0.17 → ^0.19, `jsonwebtoken` ^9 → ^10,
  `md5` ^0.7 → ^0.8, `oneshot` ^0.1 → ^0.2, `flume` ^0.11 → ^0.12,
  `strum` ^0.26 → ^0.28. Add `moka` ^0.12 (BinaryMD5 cache).

- **Strip all `aws-*` crates from the dependency graph.** The workspace now
  pins `rustls` with `default-features = false` and only the `ring` provider;
  `reqwest` uses `rustls-no-provider`. Verified: no `aws-` package remains in
  `cargo tree`.

- **sea-orm** upgraded to `^2.0.0-rc`. The `SafeEntityTrait` macro and all 33
  business call sites have been ported to the new `ValidatedUpdateOne` API.
  `strum` bumped to ^0.28 to match.

- **minio** upgraded to `^0.4`. `Client` → `MinioClientBuilder`, bucket
  provisioning uses `.bucket_exists()?.build().send()` + `S3Api` trait.

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

- Resync the zhs/en Java-sync roadmaps with the actual domain status:
  both versions now carry the same Status column, with the remaining
  gaps (score field-level diff, RSA/JWKS rotation) stated explicitly.

### Known gaps (unchanged from the Java reference)

- Score `do_generate_score`: simplified aggregation (counts edits per
  contributor). Java's full field-level diff algorithm
  (`ScoreDataPunctuateVo`) is not yet ported.

- Tokens are HMAC-SHA256; the Java side uses an RSA keypair with JWK
  rotation. The JWKS endpoint currently publishes the HMAC key in
  `oct` form.

- Database schema deviations from the real database await data
  validation (`marker_linkage` nullable columns, `sys_user_archive`
  structure binding).

- Only `docs/en` and `docs/zhs` are complete; the other 9 languages
  are skeletons.

### Notes

- The commit messages on `master` prior to the hook are a mix of Chinese and
  gitmoji; from the hook-install commit forward, all new commits follow the
  org gitmoji convention (English subject line).

- The `noa` co-author hook is reserved and not installed yet — it requires a
  built `noa` binary and the entelecheia chat-log/aporia configuration, neither
  of which is present in this repo's environment.
