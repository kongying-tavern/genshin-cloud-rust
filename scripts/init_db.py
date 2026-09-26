#!/usr/bin/env python3
"""Initialize the local Postgres schema for the Rust backend.

Wraps `cargo run --bin init_db` (the idempotent sea-orm schema initializer):
reads DB_* settings from .env / environment and passes them through, so the
tables are always generated from the current entity definitions. The bin also
applies the idempotent performance indexes from `scripts/indexes_dev.sql` (for
production, run that file manually once).

Usage:
    python scripts/init_db.py
    python scripts/init_db.py --check

All arguments are passed through to the bin unchanged (`--` is added for
cargo), so `--check` (entity ↔ database schema drift check, exit code 1 on
missing columns) works without further wrapping.

Exits non-zero when Postgres is unreachable, the schema step fails, or the
wrapped bin exits non-zero (e.g. drift detected with --check).
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent


def load_dotenv() -> None:
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


def main() -> int:
    load_dotenv()

    host = os.environ.get("DB_HOST", "localhost")
    port = os.environ.get("DB_PORT", "5432")
    user = os.environ.get("DB_USERNAME", "genshin_map")
    database = os.environ.get("DB_DATABASE", "genshin_map")
    print(f"Initializing schema at postgres://{user}@{host}:{port}/{database} ...")

    # "--" separates cargo's own flags from the bin's arguments; extra CLI
    # args are forwarded verbatim so `--check` reaches the bin.
    result = subprocess.run(
        ["cargo", "run", "--bin", "init_db", "--", *sys.argv[1:]],
        cwd=str(REPO_ROOT),
        capture_output=True,
        encoding="utf-8",
        errors="replace",
        timeout=300,
    )
    if result.returncode != 0:
        print(result.stdout[-4000:], file=sys.stdout)
        print(result.stderr[-4000:], file=sys.stderr)
        print("Schema initialization FAILED.", file=sys.stderr)
        return result.returncode

    print(result.stdout.strip())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
