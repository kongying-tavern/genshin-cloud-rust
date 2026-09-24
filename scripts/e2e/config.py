"""Shared configuration for e2e orchestration scripts.

Handles both Windows-native and WSL execution. When running under WSL,
Windows paths in .env (D:\\...) are auto-converted to /mnt/d/... via wslpath.
"""

import os
import subprocess
import urllib.parse
import urllib.request
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
            "  E2E_VUE_FRONTEND=D:\\code\\map_register_v3\n"
            "  E2E_VUE_FRONTEND=../map_register_v3   # relative to the repo root"
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

# Log files are fixed constants under STATE_DIR: never built from external
# input, so a stray name can't escape the state directory.
RUST_LOG_FILE = STATE_DIR / "rust.log"
VUE_LOG_FILE = STATE_DIR / "vue.log"
SHIRABE_LOG_FILE = STATE_DIR / "shirabe.log"

# Log file names used by the e2e child processes. Fixed constants — never
# external input; the resolved paths are verified to stay under STATE_DIR
# when opened (see open_state_log).
RUST_LOG_NAME = "rust.log"
VUE_LOG_NAME = "vue.log"
SHIRABE_LOG_NAME = "shirabe.log"


def open_state_log(name: str):
    """Open a child-process log file under STATE_DIR for writing.

    The name is a call-site constant and the resolved path is verified to
    stay inside the state directory before the file is created, so a stray
    name can't walk the log write outside it.
    """
    resolved = (STATE_DIR / name).resolve()
    if resolved.parent != STATE_DIR.resolve():
        raise ValueError(f"log path escapes the e2e state directory: {name!r}")
    fd = os.open(resolved, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o644)
    return os.fdopen(fd, "w", encoding="utf-8")


# ── Outgoing request guard ───────────────────────────────────────────────────

# The only hosts the e2e harness may talk to (the local dev services).
ALLOWED_URL_BASES = (RUST_URL, VUE_URL, SHIRABE_URL)

# Hosts a request may target after validation — the dev services are all
# loopback, and the harness must never reach any other origin.
LOCAL_HOSTS = frozenset({"127.0.0.1", "localhost", "::1"})


def guarded_url(base: str, path: str) -> str:
    """Build an outgoing request URL pinned to the configured local target.

    All HTTP calls in the e2e scripts go through here: the base must be one
    of the configured loopback services and the path a rooted string, so a
    mistyped base or an interpolated path can't turn into a request to some
    other host (SSRF hygiene for a test harness).
    """
    if base not in ALLOWED_URL_BASES:
        raise ValueError(f"base {base!r} is not an allowed e2e target")
    if not path.startswith("/"):
        raise ValueError(f"request path must start with '/': {path!r}")
    url = f"{base}{path}"
    if urllib.parse.urlsplit(url).hostname != urllib.parse.urlsplit(base).hostname:
        raise ValueError(f"URL {url!r} escapes base {base!r}")
    return url


class _NoRedirect(urllib.request.HTTPRedirectHandler):
    """Refuse redirects: a 3xx from a dev service is an error, not a hop."""

    def redirect_request(self, req, fp, code, msg, headers, newurl):  # noqa: ANN001
        return None


_opener = urllib.request.build_opener(_NoRedirect)


def local_fetch(target, timeout: float = 30):
    """Open an HTTP request whose host must be a configured loopback service.

    Every outgoing call in the e2e scripts goes through here: the target host
    is checked against the loopback allow-list at the request boundary and
    redirects are disabled, so the harness cannot be made to request an
    arbitrary origin.
    """
    url = target.full_url if isinstance(target, urllib.request.Request) else target
    if urllib.parse.urlsplit(url).hostname not in LOCAL_HOSTS:
        raise ValueError(f"refusing non-local request target: {url!r}")
    return _opener.open(target, timeout=timeout)
