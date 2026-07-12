"""Shared configuration for e2e orchestration scripts.

All paths and ports are env-var driven with sensible fallbacks.
The Vue3 frontend can be specified via E2E_VUE_FRONTEND, auto-discovered
as a sibling directory, or cloned from git if neither works.
"""

import os
import subprocess
from pathlib import Path

# ── Paths ────────────────────────────────────────────────────────────────────

# This script lives at <repo>/scripts/e2e/config.py
REPO_ROOT = Path(__file__).resolve().parent.parent.parent

# Rust backend binary name
RUST_BIN = "_router"

# ── Vue frontend path resolution ─────────────────────────────────────────────
#
# Precedence:
#   1. E2E_VUE_FRONTEND env var (absolute path)
#   2. Sibling directory auto-discovery (parent / "vue_map_register_v3")
#   3. If not found and E2E_VUE_GIT is set, clone it

E2E_VUE_GIT = os.environ.get(
    "E2E_VUE_GIT", "https://github.com/kongying-tavern/vue_map_register_v3.git"
)


def _resolve_vue_frontend() -> Path:
    # 1. Explicit env override
    env_path = os.environ.get("E2E_VUE_FRONTEND")
    if env_path:
        p = Path(env_path).resolve()
        if (p / "package.json").exists():
            return p
        raise FileNotFoundError(
            f"E2E_VUE_FRONTEND={env_path} does not contain package.json"
        )

    # 2. Auto-discover sibling directories
    candidates = [
        REPO_ROOT.parent / "vue_map_register_v3",
        REPO_ROOT.parent / "vue_map_register",
    ]
    for c in candidates:
        if (c / "package.json").exists():
            return c.resolve()

    # 3. Clone from git into target/e2e/vue_frontend
    clone_dir = REPO_ROOT / "target" / "e2e" / "vue_frontend"
    if not (clone_dir / "package.json").exists():
        print(f"📦 Cloning Vue frontend from {E2E_VUE_GIT}...")
        clone_dir.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(
            ["git", "clone", "--depth", "1", E2E_VUE_GIT, str(clone_dir)],
            check=True,
        )
    return clone_dir.resolve()


VUE_FRONTEND = _resolve_vue_frontend()

# ── Ports ────────────────────────────────────────────────────────────────────

RUST_PORT = int(os.environ.get("E2E_RUST_PORT", "8101"))
VUE_PORT = int(os.environ.get("E2E_VUE_PORT", "9000"))
SHIRABE_PORT = int(os.environ.get("E2E_SHIRABE_PORT", "3100"))

# ── URLs ─────────────────────────────────────────────────────────────────────

RUST_URL = f"http://127.0.0.1:{RUST_PORT}"
VUE_URL = f"http://127.0.0.1:{VUE_PORT}"
SHIRABE_URL = f"http://127.0.0.1:{SHIRABE_PORT}"

# ── Process state directory ──────────────────────────────────────────────────

STATE_DIR = REPO_ROOT / "target" / "e2e"
STATE_DIR.mkdir(parents=True, exist_ok=True)
PID_FILE = STATE_DIR / "processes.json"
