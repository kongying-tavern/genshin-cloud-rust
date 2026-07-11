# Changelog

All notable changes to the 空荧酒馆·原神地图 Rust backend (Genshin Map Cloud Rust)
will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

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
