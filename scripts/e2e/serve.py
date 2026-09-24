#!/usr/bin/env python3
"""E2E service orchestrator — manages Rust backend + Vue frontend.

Usage:
    python scripts/e2e/serve.py start    # launch services, then block (foreground)
    python scripts/e2e/serve.py start --daemon  # launch services, return immediately
    python scripts/e2e/serve.py stop     # stop all services
    python scripts/e2e/serve.py status   # print JSON status
    python scripts/e2e/serve.py restart  # stop + start
"""

from __future__ import annotations

import json
import os
import shutil
import signal
import subprocess
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from config import (  # noqa: E402
    PID_FILE,
    REPO_ROOT,
    RUST_LOG_NAME,
    RUST_PORT,
    RUST_URL,
    VUE_FRONTEND,
    VUE_LOG_NAME,
    VUE_PORT,
    VUE_URL,
    guarded_url,
    local_fetch,
    open_state_log,
)
from log import info, warn, error  # noqa: E402

TARGET = "e2e::serve"

# ── Helpers ──────────────────────────────────────────────────────────────────


def _load_env() -> dict[str, str]:
    env_file = REPO_ROOT / ".env"
    env: dict[str, str] = {}
    if env_file.exists():
        for line in env_file.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            if "=" in line:
                k, v = line.split("=", 1)
                env[k.strip()] = v.strip()
    return env


def _wait_for_http(base: str, path: str = "/", timeout: float = 60) -> bool:
    import urllib.error

    # Pin the probe to the configured local service; any other origin is a
    # configuration bug and fails fast via guarded_url.
    url = guarded_url(base, path)
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            local_fetch(url, timeout=3)
            return True
        except urllib.error.HTTPError:
            return True
        except Exception:
            time.sleep(1)
    return False


def _save_state(processes: dict) -> None:
    PID_FILE.write_text(json.dumps(processes, indent=2), encoding="utf-8")


def _load_state() -> dict:
    if PID_FILE.exists():
        return json.loads(PID_FILE.read_text(encoding="utf-8"))
    return {}


def _kill_proc(pid: int) -> bool:
    try:
        if os.name == "nt":
            subprocess.run(
                ["taskkill", "/PID", str(pid), "/T", "/F"],
                capture_output=True,
                timeout=10,
            )
        else:
            os.kill(pid, signal.SIGTERM)
            time.sleep(2)
            try:
                os.kill(pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
        return True
    except Exception:
        return False


# ── Service definitions ──────────────────────────────────────────────────────


def start_rust() -> dict | None:
    """Start the Rust backend.

    If malkuth is available, wrap cargo run with file-watch + auto-restart.
    malkuth's proxy listens on RUST_PORT and forwards to the pod, which gets
    a port from the proxy range via the PORT env var. On file change malkuth
    drains and restarts the pod (hot reload).
    """
    env = {**os.environ, **_load_env()}

    malkuth = shutil.which("malkuth")
    if malkuth:
        # malkuth proxy on RUST_PORT, pods assigned from RUST_PORT+1..RUST_PORT+9
        pod_port_lo = RUST_PORT + 1
        pod_port_hi = RUST_PORT + 9
        info(TARGET, f"Starting Rust backend via malkuth (proxy:{RUST_PORT} pods:{pod_port_lo}-{pod_port_hi})...")
        # Remove PORT from env so malkuth can assign it via --port-env
        env.pop("PORT", None)
        cmd = [
            malkuth,
            "--watch", str(REPO_ROOT / "packages"),
            "--proxy", f"{RUST_PORT}:{pod_port_lo}-{pod_port_hi}",
            "--port-env", "PORT",
            "--",
            "cargo", "run", "--bin", "_router",
        ]
    else:
        info(TARGET, f"Starting Rust backend on port {RUST_PORT}...")
        env["PORT"] = str(RUST_PORT)
        cmd = ["cargo", "run", "--bin", "_router"]

    proc = subprocess.Popen(
        cmd,
        cwd=str(REPO_ROOT),
        env=env,
        stdout=open_state_log(RUST_LOG_NAME),
        stderr=subprocess.STDOUT,
    )

    if _wait_for_http(RUST_URL, timeout=120):
        info(TARGET, f"Rust backend ready at {RUST_URL}")
        return {"pid": proc.pid, "url": RUST_URL, "name": "rust"}
    else:
        error(TARGET, f"Rust backend did not become ready in 120s")
        _kill_proc(proc.pid)
        return None


def start_vue() -> dict | None:
    info(TARGET, f"Starting Vue dev server on port {VUE_PORT}...")
    # Resolve pnpm to an absolute path (pnpm.cmd on Windows) so the child can
    # be spawned as a plain argv list without going through the shell.
    pnpm = shutil.which("pnpm")
    if not pnpm:
        error(TARGET, "pnpm not found on PATH — cannot start Vue dev server")
        return None
    proc = subprocess.Popen(
        [pnpm, "dev"],
        cwd=str(VUE_FRONTEND),
        stdout=open_state_log(VUE_LOG_NAME),
        stderr=subprocess.STDOUT,
    )

    if _wait_for_http(VUE_URL, timeout=60):
        info(TARGET, f"Vue frontend ready at {VUE_URL}")
        return {"pid": proc.pid, "url": VUE_URL, "name": "vue"}
    else:
        error(TARGET, f"Vue frontend did not become ready in 60s")
        _kill_proc(proc.pid)
        return None


# ── Commands ─────────────────────────────────────────────────────────────────


def cmd_start(daemon: bool = False) -> int:
    setup_script = Path(__file__).parent / "setup_frontend.py"
    subprocess.run([sys.executable, str(setup_script)], check=True)

    processes = {}

    rust_info = start_rust()
    if rust_info is None:
        return 1
    processes["rust"] = rust_info

    vue_info = start_vue()
    if vue_info is None:
        _kill_proc(rust_info["pid"])
        return 1
    processes["vue"] = vue_info

    _save_state(processes)
    info(TARGET, f"All services running — Rust: {RUST_URL}  Vue: {VUE_URL}")

    if daemon:
        return 0

    # Foreground mode: block until Ctrl-C, then clean up.
    info(TARGET, "Running in foreground. Press Ctrl-C to stop all services.")
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        info(TARGET, "Received Ctrl-C, stopping all services...")
        cmd_stop()
        return 0


def cmd_stop() -> int:
    state = _load_state()
    if not state:
        info(TARGET, "No services recorded.")
        return 0

    for name, svc_info in state.items():
        pid = svc_info.get("pid")
        if pid:
            killed = _kill_proc(pid)
            status = "killed" if killed else "already dead"
            info(TARGET, f"{name} (pid {pid}): {status}")

    PID_FILE.unlink(missing_ok=True)
    info(TARGET, "All services stopped.")
    return 0


def cmd_status() -> int:
    state = _load_state()
    if not state:
        print(json.dumps({"running": False}))
        return 0

    alive = {}
    for name, svc_info in state.items():
        pid = svc_info.get("pid", 0)
        try:
            if os.name == "nt":
                subprocess.run(
                    ["tasklist", "/FI", f"PID eq {pid}"],
                    capture_output=True,
                    timeout=5,
                )
                alive[name] = svc_info
            else:
                os.kill(pid, 0)
                alive[name] = svc_info
        except Exception:
            pass

    output = {
        "running": len(alive) > 0,
        "services": alive,
        "rust_url": RUST_URL,
        "vue_url": VUE_URL,
    }
    print(json.dumps(output, indent=2))
    return 0 if alive else 1


def main() -> int:
    args = sys.argv[1:]
    if not args or args[0] == "start":
        daemon = "--daemon" in args
        return cmd_start(daemon=daemon)
    elif args[0] == "stop":
        return cmd_stop()
    elif args[0] == "status":
        return cmd_status()
    elif args[0] == "restart":
        cmd_stop()
        time.sleep(2)
        return cmd_start()
    else:
        error(TARGET, f"Unknown command: {args[0]}")
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
