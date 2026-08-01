# Genshin Map Cloud (Rust) justfile
#
# Verb-first dispatch: every recipe is a VERB (build, dev, test, ...).
# No noun-only or namespace-style commands.
#
#   just build         # build the router (release)
#   just build --dev   # debug build
#   just dev           # start Rust + Vue dev stack
#   just dev mock      # start + Shirabe e2e tests + stop
#   just dev stop      # stop dev stack
#   just test          # run cargo tests
#   just ci            # fmt-check + clippy + check + test
#   just fmt           # format code + docs

set unstable
set lists
# Use the full path to Git Bash to avoid WSL bash hijacking.
set windows-shell := ["C:/Program Files/Git/usr/bin/bash.exe", "-c"]

# Import vendored devtools recipes (provides python_cmd, cache-guard, etc.).
# Recipes we don't use are overridden below or simply ignored.
import "./celestia-devtools.just"

default:
    @just --list

# ── init ─────────────────────────────────────────────────────────────────────

# Initialize the development environment (devtools + cargo fetch + hooks).
init:
    @echo "🔧 Initializing development environment..."
    celestia-devtools init
    cargo fetch
    celestia-devtools hook install --force
    @echo "✨ Initialization complete!"

# Install the celestia-devtools commit-msg hook (gitmoji convention).
hooks:
    celestia-devtools hook install --force
    @echo "✅ commit-msg hook installed"

# ── build ────────────────────────────────────────────────────────────────────

# Build the router. Release by default; --dev for debug, --clean to clean first.
build *FLAGS='':
    just _build ":" "cargo build" "cargo build --release" {{FLAGS}}

# Type-check the workspace without producing binaries.
check:
    cargo check --workspace --all-targets

# Remove build artifacts.
clean:
    cargo clean

# ── format & lint ────────────────────────────────────────────────────────────

# Format Rust code + Markdown docs.
fmt:
    cargo fmt --all
    {{ python_cmd }} -m celestia_devtools format-markdown . || true

# Check formatting without modifying files.
fmt-check:
    cargo fmt --all -- --check

# Run clippy with strict warnings.
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# ── test ─────────────────────────────────────────────────────────────────────

# Run the workspace test suite.
test:
    cargo test --workspace --all-targets --no-fail-fast

# ── ci ───────────────────────────────────────────────────────────────────────

# Full CI gate: format check + clippy + compile check + test.
ci: fmt-check clippy check test

# ── dev ──────────────────────────────────────────────────────────────────────

# Start the Rust backend only (reads .env for DB/host config).
run *ARGS:
    cargo run --bin _router -- {{ARGS}}

# Dev mode: start Rust backend + Vue3 frontend together.
#   just dev              # start both, block in foreground (Ctrl-C to stop)
#   just dev daemon       # start both, return immediately (use `just dev stop`)
#   just dev mock         # start → Shirabe browser e2e tests → stop
#   just dev stop         # stop both
#   just dev status       # check status
#   just dev restart      # stop + start
dev *ARGS='':
    {{ python_cmd }} scripts/e2e/dev.py {{ARGS}}

