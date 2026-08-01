#!/usr/bin/env python3
"""Dev orchestrator — thin CLI wrapper around serve.py + run_tests.py.

Usage:
    just dev              # start both services, block in foreground (Ctrl-C to stop)
    just dev daemon       # start both services, return immediately (use `just dev stop`)
    just dev mock         # start → Shirabe e2e tests → stop
    just dev stop         # stop all
    just dev status       # print JSON status
    just dev restart      # stop + start
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from log import info, error  # noqa: E402

SCRIPTS_DIR = Path(__file__).resolve().parent
SERVE_PY = SCRIPTS_DIR / "serve.py"
RUN_TESTS_PY = SCRIPTS_DIR / "run_tests.py"
SETUP_PY = SCRIPTS_DIR / "setup_frontend.py"

TARGET = "e2e::dev"


def _run(script: Path, *args: str) -> int:
    return subprocess.call([sys.executable, str(script), *args])


def main() -> int:
    cmd = sys.argv[1] if len(sys.argv) > 1 else ""

    if cmd == "" or cmd == "start":
        info(TARGET, "Starting dev stack (Rust + Vue), foreground mode...")
        _run(SETUP_PY)
        return _run(SERVE_PY, "start")

    elif cmd == "daemon":
        info(TARGET, "Starting dev stack (Rust + Vue), daemon mode...")
        _run(SETUP_PY)
        return _run(SERVE_PY, "start", "--daemon")

    elif cmd == "mock":
        info(TARGET, "Starting dev stack + Shirabe e2e tests...")
        _run(SETUP_PY)
        rc = _run(SERVE_PY, "start", "--daemon")
        if rc != 0:
            return rc
        rc = _run(RUN_TESTS_PY)
        _run(SERVE_PY, "stop")
        return rc

    elif cmd == "stop":
        return _run(SERVE_PY, "stop")

    elif cmd == "status":
        return _run(SERVE_PY, "status")

    elif cmd == "restart":
        _run(SERVE_PY, "stop")
        return _run(SERVE_PY, "start")

    else:
        error(TARGET, f"Unknown command: {cmd}. Usage: just dev [daemon|mock|stop|status|restart]")
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
