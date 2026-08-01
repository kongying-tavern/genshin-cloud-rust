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
RUN mkdir -p packages/utils/src packages/database/src packages/functions/src \
        packages/router/src tests/rust/src \
 && printf 'pub fn _stub() {}\n' > packages/utils/src/lib.rs \
 && printf 'pub fn _stub() {}\n' > packages/database/src/lib.rs \
 && printf 'pub fn _stub() {}\n' > packages/functions/src/lib.rs \
 && printf 'fn main() {}\n'        > packages/router/src/main.rs \
 && printf ''                       > tests/rust/src/lib.rs \
 && cargo fetch --locked

# Real sources + release build. The cache mounts keep cargo registry and the
# target dir out of the image layers; the binary is copied to a stable path so
# it survives the cache mount being unmounted.
COPY packages/ packages/
COPY tests/     tests/
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    cargo build --release --package _router \
 && cp target/release/_router /usr/local/bin/_router

# ── Runtime ──────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

# ca-certificates: reqwest loads the system root store via rustls-native-certs
# for outbound HTTPS (CDN proxy, OAuth, etc.).
# tini: reaps zombies and forwards SIGTERM to tokio for graceful shutdown.
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        tini \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /usr/local/bin/_router /usr/local/bin/_router

# The router listens on port 80 by default (see packages/router/src/main.rs);
# override with the PORT env var if needed.
ENV RUST_LOG=info
EXPOSE 80

ENTRYPOINT ["/usr/bin/tini", "--", "_router"]
