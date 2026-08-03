#!/usr/bin/env python3
"""Seed the local database with a small set of demo map data.

The real map dataset lives in the production database; a fresh local DB is
empty, so the map renders pitch black. This script inserts a handful of
areas, items, icons and ~150 spread-out markers (Mondstadt-ish coordinates)
so the frontend has something to show.

Usage:
    python scripts/seed_demo.py

Idempotent: wipes the demo-scoped business tables first, then re-inserts.
DB connection comes from .env / environment (same DB_* vars as init_db).
"""

from __future__ import annotations

import os
import random
import sys
from pathlib import Path

import psycopg2

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
    host = os.environ.get("DB_HOST", "127.0.0.1")
    port = int(os.environ.get("DB_PORT", "5432"))
    user = os.environ.get("DB_USERNAME", "genshin_map")
    password = os.environ.get("DB_PASSWORD", "")
    database = os.environ.get("DB_DATABASE", "genshin_map")

    try:
        conn = psycopg2.connect(
            host=host, port=port, user=user, password=password, dbname=database
        )
    except Exception as e:
        print(f"DB connect failed ({host}:{port}/{database}): {e}", file=sys.stderr)
        return 1
    conn.autocommit = True
    cur = conn.cursor()

    # ── Wipe the demo-scoped business tables (not sys_user etc.) ────────────
    for table in (
        "marker_punctuate",
        "marker_item_link",
        "marker",
        "history",
        "item_area_public",
        "item",
        "icon",
        "area",
    ):
        cur.execute(f'DELETE FROM "genshin_map"."{table}"')

    now = "now()"

    # ── Areas (root area self-references its parent_id) ──────────────────────
    areas = [
        (1, "蒙德", 1),
        (2, "璃月", 1),
        (3, "稻妻", 1),
    ]
    for aid, name, parent in areas:
        cur.execute(
            f'INSERT INTO "genshin_map"."area" '
            f"(version, id, create_time, update_time, creator_id, updater_id, del_flag, "
            f"name, code, content, icon_tag, parent_id, is_final, hidden_flag, sort_index, special_flag) "
            f"VALUES (0, {aid}, {now}, NULL, NULL, NULL, false, %s, NULL, NULL, '0', {parent}, true, 0, {aid}, 0)",
            (name,),
        )

    # ── Icons ────────────────────────────────────────────────────────────────
    icons = [
        (1, "锚点", "https://v3.yuanshen.site/icon/Anchor.png"),
        (2, "神像", "https://v3.yuanshen.site/icon/Statue.png"),
        (3, "秘境", "https://v3.yuanshen.site/icon/Domain.png"),
        (4, "宝箱", "https://v3.yuanshen.site/icon/Chest.png"),
        (5, "材料", "https://v3.yuanshen.site/icon/Item.png"),
    ]
    for iid, name, url in icons:
        cur.execute(
            f'INSERT INTO "genshin_map"."icon" '
            f"(version, id, create_time, update_time, creator_id, updater_id, del_flag, name, url) "
            f"VALUES (0, {iid}, {now}, NULL, NULL, NULL, false, %s, %s)",
            (name, url),
        )

    # ── Items ────────────────────────────────────────────────────────────────
    items = [
        (1, "传送锚点", 1, "0", 0),
        (2, "七天神像", 1, "0", 0),
        (3, "秘境", 2, "0", 0),
        (4, "珍贵宝箱", 1, "0", 0),
        (5, "普通宝箱", 1, "0", 0),
        (6, "松茸", 1, "0", 0),
        (7, "薄荷", 1, "0", 0),
    ]
    for iid, name, area_id, icon_tag, sort in items:
        cur.execute(
            f'INSERT INTO "genshin_map"."item" '
            f"(version, id, create_time, update_time, creator_id, updater_id, del_flag, "
            f"name, area_id, default_refresh_time, default_content, default_count, "
            f"icon_tag, icon_style_type, hidden_flag, sort_index, special_flag) "
            f"VALUES (0, {iid}, {now}, NULL, NULL, NULL, false, %s, {area_id}, 0, NULL, 1, %s, 0, 0, {sort}, NULL)",
            (name, icon_tag),
        )

    # ── Markers: ~150 points spread over the Mondstadt region ────────────────
    # Teyvat coordinates; Mondstadt city sits around (2400, 1900).
    rng = random.Random(20260803)
    marker_id = 1
    title_by_item = {
        1: "传送锚点",
        2: "七天神像",
        3: "秘境",
        4: "珍贵宝箱",
        5: "普通宝箱",
        6: "松茸",
        7: "薄荷",
    }
    for _ in range(150):
        item_id = rng.choice(list(title_by_item))
        x = rng.randint(1800, 3200)
        y = rng.randint(1200, 2600)
        position = f"{x}.{rng.randint(0, 9)},{y}.{rng.randint(0, 9)}"
        cur.execute(
            f'INSERT INTO "genshin_map"."marker" '
            f"(version, id, create_time, update_time, creator_id, updater_id, del_flag, "
            f"marker_stamp, marker_title, position, content, picture, marker_creator_id, "
            f"picture_creator_id, video_path, refresh_time, hidden_flag, extra) "
            f"VALUES (0, {marker_id}, {now}, NULL, NULL, NULL, false, NULL, %s, %s, '', NULL, 1, NULL, NULL, 0, 0, NULL)",
            (title_by_item[item_id], position),
        )
        cur.execute(
            f'INSERT INTO "genshin_map"."marker_item_link" '
            f"(version, id, create_time, update_time, creator_id, updater_id, del_flag, "
            f"item_id, marker_id, count) "
            f"VALUES (0, {marker_id}, {now}, NULL, NULL, NULL, false, {item_id}, {marker_id}, 1)"
        )
        marker_id += 1

    cur.execute('SELECT count(*) FROM "genshin_map"."marker"')
    marker_count = cur.fetchone()[0]
    cur.execute('SELECT count(*) FROM "genshin_map"."item"')
    item_count = cur.fetchone()[0]
    cur.execute('SELECT count(*) FROM "genshin_map"."area"')
    area_count = cur.fetchone()[0]

    conn.close()
    print(f"Demo data seeded: {area_count} areas, {item_count} items, {marker_count} markers")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
