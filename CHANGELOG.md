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
