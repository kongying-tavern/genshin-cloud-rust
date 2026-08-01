"""Structured logging for e2e scripts, matching the Rust backend's tracing format.

Rust tracing output looks like:
  [2026-07-13T02:06:46Z INFO  _router] Site will run on port 8101

Python scripts match this:
  [2026-07-13T10:30:00Z INFO  e2e::serve] Starting Rust backend on port 8101...
"""

from __future__ import annotations

import sys
from datetime import datetime, timezone


def _format(level: str, target: str, msg: str) -> str:
    ts = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    # tracing pads level to 5 chars right-justified, target left-justified
    return f"[{ts} {level:<5} {target}] {msg}"


def info(target: str, msg: str) -> None:
    print(_format("INFO", target, msg), file=sys.stdout, flush=True)


def warn(target: str, msg: str) -> None:
    print(_format("WARN", target, msg), file=sys.stdout, flush=True)


def error(target: str, msg: str) -> None:
    print(_format("ERROR", target, msg), file=sys.stderr, flush=True)
