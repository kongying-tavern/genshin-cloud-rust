# Genshin Map Cloud (Rust) justfile
#
# Verb-first dispatch: the first word is always a VERB (build, run, test,
# clean, fmt, gen, ...). Targets are positional args dispatched via `case`.
#   just test rust         # run rust tests
#   just build --dev       # debug build
#   just ci                # fmt-check + clippy + check + test

set unstable
set lists
# Git for Windows keeps bash.exe on PATH; cygpath is NOT on PATH, so shebang
# recipes die without this. The `windows-shell-check` recipe (imported below)
# verifies it.
set shell := ["bash", "-c"]
set windows-shell := ["bash.exe", "-c"]

import "./celestia-devtools.just"

default:
    @just --list

# ── Hooks ────────────────────────────────────────────────────────────────────

# Install (or refresh) the celestia-devtools commit-msg hook locally.
# This enforces the org gitmoji convention on every `git commit`.
# Run once per fresh checkout. (noa hook is reserved for when noa is built
# and configured — see docs/en/guides/commit-message-convention.md.)
hooks:
    celestia-devtools hook install --force
    @echo "✅ commit-msg hook installed (gitmoji convention enforced)"

# ── Initialization ───────────────────────────────────────────────────────────

# Initialize the development environment.
init:
    @echo "🔧 Initializing development environment..."
    celestia-devtools init
    cargo fetch
    @echo "✨ Initialization complete! Run 'just hooks' to install the commit-msg hook."

# ── Build ────────────────────────────────────────────────────────────────────

# Build the router. Release by default; `--dev` for debug, `--clean` first.
build *FLAGS='':
    just _build ":" "cargo build" "cargo build --release" {{FLAGS}}

check:
    cargo check --workspace --all-targets

clean:
    cargo clean

# ── Format & Lint ────────────────────────────────────────────────────────────

fmt:
    cargo fmt --all
    {{ python_cmd }} -m celestia_devtools format-markdown . || true

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# ── Test ─────────────────────────────────────────────────────────────────────

test:
    cargo test --workspace --all-targets --no-fail-fast

# ── CI ───────────────────────────────────────────────────────────────────────

ci: fmt-check clippy check test

# ── Run ──────────────────────────────────────────────────────────────────────

run *ARGS:
    cargo run --bin _router -- {{ARGS}}
