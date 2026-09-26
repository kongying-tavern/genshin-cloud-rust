# syntax=docker/dockerfile:1.7

# Genshin Map Cloud — Rust backend image.
#
# Multi-stage: a `rust:1` builder compiles the `_router` binary with LTO, then a
# `debian:bookworm-slim` runtime ships only the binary + ca-certificates + tini.
# BuildKit cache mounts (registry + target) keep rebuilds fast.

# ── Builder ──────────────────────────────────────────────────────────────────
FROM rust:1-bookworm AS builder

WORKDIR /app

# Copy only manifests + lockfile first so the dependency-fetch layer is cached
# independently of source changes.
COPY Cargo.toml Cargo.lock ./
COPY packages/utils/Cargo.toml      packages/utils/Cargo.toml
COPY packages/database/Cargo.toml   packages/database/Cargo.toml
COPY packages/functions/Cargo.toml  packages/functions/Cargo.toml
COPY packages/router/Cargo.toml     packages/router/Cargo.toml
COPY tests/rust/Cargo.toml          tests/rust/Cargo.toml

# Stub member sources so cargo can resolve the workspace without the real code,
# then fetch all dependencies. The stubs are overwritten when real sources land.
# The bench stub is required too: cargo validates [[bench]] target paths when
# parsing the manifest, so `cargo fetch` fails without a file there. The router
# lib stub follows the same rule — the crate is lib+bin, so its [lib] target
# path must exist before the real sources are copied in.
RUN mkdir -p packages/utils/src packages/database/src packages/functions/src \
        packages/functions/benches packages/router/src tests/rust/src \
 && printf 'pub fn _stub() {}\n' > packages/utils/src/lib.rs \
 && printf 'pub fn _stub() {}\n' > packages/database/src/lib.rs \
 && printf 'pub fn _stub() {}\n' > packages/functions/src/lib.rs \
 && printf 'fn main() {}\n' > packages/functions/benches/diff_snapshot.rs \
 && printf 'fn main() {}\n'        > packages/router/src/main.rs \
 && printf 'pub fn _stub() {}\n' > packages/router/src/lib.rs \
 && printf ''                       > tests/rust/src/lib.rs \
 && cargo fetch --locked

# Real sources + release build. The cache mounts keep cargo registry and the
# target dir out of the image layers; the binary is copied to a stable path so
# it survives the cache mount being unmounted.
# indexes_dev.sql is embedded by the init_db bin via include_str!.
COPY packages/                     packages/
COPY tests/                        tests/
COPY scripts/indexes_dev.sql       scripts/indexes_dev.sql
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    cargo build --release --package _router \
 && cp target/release/_router /usr/local/bin/_router

# ── Runtime ──────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

# ca-certificates: reqwest loads the system root store via rustls-native-certs
# for outbound HTTPS (CDN proxy, OAuth, etc.).
# tini: reaps zombies and forwards SIGTERM to tokio for graceful shutdown.
# wget: container healthcheck (jwks probe).
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        tini \
        wget \
    && rm -rf /var/lib/apt/lists/*

# Non-root runtime user. LOG_DIR 若在容器内启用，目录需对该 UID 可写。
RUN groupadd --system --gid 10001 app && useradd --system --uid 10001 --gid app --no-create-home app
USER app

WORKDIR /app
COPY --from=builder /usr/local/bin/_router /usr/local/bin/_router

# The router listens on port 80 by default (see packages/router/src/main.rs);
# override with the PORT env var if needed.
ENV RUST_LOG=info
EXPOSE 80

# Pure-compute probe: JWKS needs no DB/Redis, so it isolates "process alive".
HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 CMD wget -q --spider "http://127.0.0.1:${PORT:-80}/.well-known/jwks.json" || exit 1

ENTRYPOINT ["/usr/bin/tini", "--", "_router"]

