# Genshin Map Cloud (Rust) justfile
#
# Verb-first dispatch: every recipe is a VERB (build, dev, test, ...).
#
#   just build         # build the router (release)
#   just dev           # start Rust + Vue dev stack
#   just test          # run cargo tests
#   just test-e2e      # run the python live-API regression suite
#   just bench         # run criterion benchmarks
#   just ci            # fmt-check + clippy + check + test
#   just fmt           # format code + docs + Cargo.toml
#
# Shared celestia-devtools recipes are staged into .just/ (gitignored) with
# `just fetch`; SHELL DOCTRINE and overrides follow the staged template.

set shell := ["bash", "-c"]
# Windows: PowerShell (the 5.1 floor ships with every Windows; pwsh 7 is
# NOT assumed). Linewise recipes must stay PS-5.1-safe: no `&&` chains,
# `cd X; cmd` instead of `cd X && cmd`. Bash-only recipes use
# [script('bash')] and need Git Bash (or WSL) when actually run.
set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command", "[Console]::OutputEncoding=[System.Text.Encoding]::UTF8; $PSDefaultParameterValues['*:Encoding']='utf8';"]
set unstable
set lists

# Repo definitions override the shared template's (imported above).
set allow-duplicate-recipes
set allow-duplicate-variables

# Fallback definitions so this justfile parses on a fresh clone where the
# staged .just/ recipes are absent (the import below is optional). The staged
# template overrides these; both spellings resolve identically.
python_cmd := if os_family() == "windows" {
    if which("python") != "" { "python" } else { "python3" }
} else {
    if which("python3") != "" { "python3" } else { "python" }
}
_devtools := if which("celestia-devtools") != "" { "celestia-devtools" } else { python_cmd + " -m celestia_devtools" }

# Shared celestia-devtools recipes — NOT in git. Stage with: just fetch.
# `import?` silently skips when absent, so this justfile parses pre-fetch.
import? "./.just/celestia-devtools.just"

# Stage shared celestia-devtools recipes into .just/ (gitignored).
# Source order: explicit URL arg → local pip bundle (offline) → GitHub raw.
# curl honors HTTP_PROXY/HTTPS_PROXY/ALL_PROXY env vars automatically.
fetch URL='':
    {{ if os_family() == "windows" { "python" } else { "python3" } }} -c "import os; os.makedirs('.just', exist_ok=True)"
    {{ if URL != "" { "curl -fsSL " + URL + " -o .just/celestia-devtools.just" } else if which("celestia-devtools") != "" { "celestia-devtools fetch-just" } else { "curl -fsSL https://raw.githubusercontent.com/celestia-island/celestia-devtools/dev/src/celestia_devtools/common.just -o .just/celestia-devtools.just" } }}

default:
    @just --list

# ── init ─────────────────────────────────────────────────────────────────────

# Initialize the development environment (devtools + cargo fetch + hooks).
# `just ensure` installs celestia-devtools when missing (pip), so a fresh
# clone needs nothing beyond python + cargo.
init:
    just ensure
    {{ _devtools }} init
    cargo fetch
    {{ _devtools }} hook install --force

# Install the celestia-devtools commit-msg hook (gitmoji convention).
hooks:
    just ensure
    {{ _devtools }} hook install --force

# ── build ────────────────────────────────────────────────────────────────────

# Build the router. Release by default; --dev for debug, --clean to clean first.
# cache-guard runs first (free-disk floor → cargo clean; oversize → sweep).
build *FLAGS='':
    just _build "just cache-guard" "cargo build" "cargo build --release" {{FLAGS}}

# Type-check the workspace without producing binaries.
check:
    cargo check --workspace --all-targets

# Remove build artifacts.
clean:
    cargo clean

# ── format & lint ────────────────────────────────────────────────────────────

# Format Rust code + Markdown docs + Cargo.toml dependency order.
fmt:
    just fmt-toml
    just fmt-markdown .
    cargo fmt --all

# Check formatting without modifying files.
fmt-check:
    just fmt-markdown . --check
    cargo fmt --all -- --check

# Run clippy with strict warnings.
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# ── test ─────────────────────────────────────────────────────────────────────

# Run the workspace test suite.
test:
    cargo test --workspace --all-targets --no-fail-fast

# Run the live-API python regression suite (tests/python).
# Prerequisites: a running backend + seeded regress_* accounts; the suite
# self-skips unless REGRESSION_PASSWORD is exported. See tests/python/README.md.
test-e2e:
    {{ python_cmd }} tests/python/run_all.py

# ── bench ────────────────────────────────────────────────────────────────────

# Run criterion benchmarks (currently: marker_doc diff snapshot pipeline).
#   just bench                                             # full run
#   just bench diff_snapshot                               # filter by name
#   just bench -- --sample-size 10 --measurement-time 2    # quick mode
# Pure CPU benches always run; the DB group self-skips like the *_db tests.
# To enable it, start the e2e Postgres (tests/docker) and run with
#   GCS_TEST_DB=1 just bench
bench *ARGS:
    cargo bench --workspace {{ARGS}}

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
