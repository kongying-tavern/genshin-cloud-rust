#!/usr/bin/env python3
"""Dev orchestrator — thin CLI wrapper around serve.py + run_tests.py.

Usage:
    python scripts/e2e/dev.py             # start Rust + Vue
    python scripts/e2e/dev.py mock        # start → Shirabe e2e tests → stop
    python scripts/e2e/dev.py stop        # stop all
    python scripts/e2e/dev.py status      # print JSON status
    python scripts/e2e/dev.py restart     # stop + start
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

SCRIPTS_DIR = Path(__file__).resolve().parent
SERVE_PY = SCRIPTS_DIR / "serve.py"
RUN_TESTS_PY = SCRIPTS_DIR / "run_tests.py"
SETUP_PY = SCRIPTS_DIR / "setup_frontend.py"


def _run(script: Path, *args: str) -> int:
    return subprocess.call([sys.executable, str(script), *args])


def main() -> int:
    cmd = sys.argv[1] if len(sys.argv) > 1 else ""

    if cmd == "" or cmd == "start":
        print("🚀 Starting dev stack (Rust + Vue)...")
        # Ensure frontend is set up
        _run(SETUP_PY)
        return _run(SERVE_PY, "start")

    elif cmd == "mock":
        print("🤖 Starting dev stack + Shirabe e2e tests...")
        _run(SETUP_PY)
        rc = _run(SERVE_PY, "start")
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
        print(f"Usage: {sys.argv[0]} [start|mock|stop|status|restart]", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
