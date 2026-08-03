"""Shared configuration for e2e orchestration scripts.

Handles both Windows-native and WSL execution. When running under WSL,
Windows paths in .env (D:\\...) are auto-converted to /mnt/d/... via wslpath.
"""

import os
import subprocess
from pathlib import Path

# ── Paths ────────────────────────────────────────────────────────────────────

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
RUST_BIN = "_router"


def _is_wsl() -> bool:
    """Detect WSL by checking /proc/version for 'microsoft'."""
    try:
        with open("/proc/version", encoding="utf-8", errors="replace") as f:
            return "microsoft" in f.read().lower()
    except Exception:
        return False


def _win_to_native(path_str: str) -> str:
    """Convert a Windows path to the native format.

    On WSL, use `wslpath -u` to convert D:\\foo → /mnt/d/foo.
    On Windows, return as-is.
    """
    if _is_wsl():
        try:
            result = subprocess.run(
                ["wslpath", "-u", path_str],
                capture_output=True, text=True, timeout=5,
            )
            if result.returncode == 0 and result.stdout.strip():
                return result.stdout.strip()
        except Exception:
            pass
    return path_str


def _load_dotenv() -> None:
    """Load .env from REPO_ROOT into os.environ (if not already set)."""
    env_file = REPO_ROOT / ".env"
    if not env_file.exists():
        return
    for line in env_file.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        k, v = line.split("=", 1)
        k, v = k.strip(), v.strip()
        if k and k not in os.environ:
            os.environ[k] = v


_load_dotenv()

# ── Vue frontend path (required in .env) ─────────────────────────────────────


def _resolve_vue_frontend() -> Path:
    env_path = os.environ.get("E2E_VUE_FRONTEND")
    if not env_path:
        raise RuntimeError(
            "E2E_VUE_FRONTEND is not set — refusing to start.\n"
            "Add it to .env, e.g.:\n"
            "  E2E_VUE_FRONTEND=D:\\code\\vue_map_register_v3\n"
            "  E2E_VUE_FRONTEND=../vue_map_register_v3   # relative to the repo root"
        )
    # Convert Windows path to native if running in WSL
    native_path = _win_to_native(env_path)
    p = Path(native_path)
    # Relative paths resolve against the repo root (not the CWD), so the
    # .env entry is stable regardless of where the script is invoked from.
    if not p.is_absolute():
        p = REPO_ROOT / p
    p = p.resolve()
    if not (p / "package.json").exists():
        raise RuntimeError(
            f"E2E_VUE_FRONTEND={env_path}\n"
            f"  Resolved to: {p}\n"
            f"  Does not contain package.json — wrong path?"
        )
    return p


VUE_FRONTEND = _resolve_vue_frontend()

# ── Ports ────────────────────────────────────────────────────────────────────

RUST_PORT = int(os.environ.get("E2E_RUST_PORT", os.environ.get("PORT", "8101")))
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
