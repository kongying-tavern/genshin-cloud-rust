# Building

How to get the backend compiling and running locally, plus the environment
variables, the local docker-compose stack, and how CI mirrors the same steps.

## Prerequisites

| Tool | Why | Notes |
| --- | --- | --- |
| Rust (stable) | Compiles the workspace. | Pinned by `rust-toolchain.toml` (stable + `rustfmt` + `clippy`). Edition 2024. |
| [`just`](https://github.com/casey/just) | Task runner; the recommended entry point. | `justfile` imports `celestia-devtools.just`. |
| Docker + `docker-compose` | Local Postgres, Redis, MinIO. | Use `dev.compose.yml`. |
| `celestia-devtools` (Python) | Commit-msg hook + markdown formatting. | Installed by `just init`. |

The TLS stack is `rustls` with the **ring** provider only — no `aws-lc-rs` /
`aws-lc-sys` C build is required, so the workspace compiles cleanly on MSVC
without a C toolchain.

## Common commands

```bash
just init          # initialize the dev environment (cargo fetch + devtools)
just hooks         # install the celestia-devtools commit-msg hook
just build         # build the workspace (release; add --dev for debug)
just build --clean # clean + release build
just dev           # real-time debug run
just check         # cargo check --workspace --all-targets
just fmt           # cargo fmt + markdown formatting
just fmt-check     # cargo fmt --check (runs in CI)
just clippy        # cargo clippy --workspace --all-targets
just test          # cargo test --workspace --all-targets --no-fail-fast
just ci            # fmt-check + clippy + check + test (the local CI mirror)
```

`just` uses verb-first dispatch (`just <verb> [target]`), so `just build`,
`just test`, `just dev` all do what you would expect.

## The `.env` file

Create a `.env` in the repo root before running the server. All variables are
read by `database::build_db_map()`; the values below match the bundled
`dev.compose.yml` stack.

```env
# PostgreSQL
DB_HOST=localhost
DB_PORT=5432
DB_USERNAME=genshin_map
DB_PASSWORD=genshin_map
DB_DATABASE=genshin_map

# Redis
REDIS_HOST=localhost
REDIS_PORT=6379
# REDIS_USERNAME=          # optional
# REDIS_PASSWORD=          # optional

# MinIO (S3-compatible)
MINIO_BASE_URL=http://localhost:9000
MINIO_ACCESS_KEY=genshin_cloud
MINIO_SECRET_KEY=genshin_cloud

# Server
# PORT=80                  # optional; defaults to 80
```

Only `DB_PASSWORD` is strictly required to start (the rest fall back to the
localhost defaults shown above). MinIO access/secret keys are required when
the object-storage path is exercised.

## Local docker-compose stack

`dev.compose.yml` brings up the three stateful services the backend needs.
This is **not** a production configuration.

```bash
docker-compose -f dev.compose.yml up -d
```

| Service | Container | Port | Credentials |
| --- | --- | --- | --- |
| PostgreSQL | `test-postgres` | `5432` | `genshin_map` / `genshin_map` / `genshin_map` (user/pass/db) |
| Redis | `test-redis` | `6379` | no auth |
| MinIO | `test-minio` | `9000` (S3), `9001` (console) | `genshin_cloud` / `genshin_cloud` |

On startup the backend auto-creates the `images` and `bz2doc` MinIO buckets
with a public `s3:GetObject` policy if they do not yet exist.

## CI

GitHub Actions mirrors the local `just ci` flow across three workflows under
`.github/workflows/`:

| Workflow | What it does |
| --- | --- |
| `rust.yml` | On push/PR to `master`/`dev`: `cargo fmt --check`, `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets`, `cargo build --workspace --release`. Uses `dtolnay/rust-toolchain@stable` + `sccache`. |
| `test.yml` | Multi-OS matrix (`ubuntu-latest`, `windows-latest`): unit tests (`--lib`) + integration tests (`--tests`); plus a `commit-msg` job that lints every commit subject against the gitmoji convention with `celestia-devtools commit-msg-lint`, and a `trufflehog` secrets scan. |
| `docs.yml` | On changes under `docs/**`: builds the multilingual site with `lagrange` and deploys to GitHub Pages. |

Note: CI runs clippy **without** `-D warnings` until the pre-existing lints
documented in `CHANGELOG.md` are resolved; `just clippy-strict` is the local
strict variant for new code.
