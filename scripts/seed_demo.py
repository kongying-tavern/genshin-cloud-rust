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
        "item_type_link",
        "item",
        "item_type",
        "tag_type_link",
        "tag_type",
        "tag",
        "notice",
        "icon",
        "area",
    ):
        cur.execute(f'DELETE FROM "genshin_map"."{table}"')

    now = "(now() AT TIME ZONE 'UTC')"

    # ── Areas (parents are is_final=false / parent_id=0; children are
    #    is_final=true under their parent. The frontend left panel shows
    #    parents in the first column and children in the second, so both
    #    kinds must exist. Codes match AREA_ADDITIONAL_CONFIG_MAP keys.) ────
    areas = [
        (1, "蒙德", 1, "A:MD:MENGDE", False),
        (11, "蒙德城", 1, "A:MD:1", True),
        (2, "璃月", 2, "A:LY:LIYUE", False),
        (12, "璃月港", 2, "A:LY:1", True),
        (3, "稻妻", 3, "A:DQ:1", False),
        (13, "稻妻城", 3, "A:DQ:2", True),
    ]
    for aid, name, parent, code, final in areas:
        cur.execute(
            f'INSERT INTO "genshin_map"."area" '
            f"(version, id, create_time, update_time, creator_id, updater_id, del_flag, "
            f"name, code, content, icon_tag, parent_id, is_final, hidden_flag, sort_index, special_flag) "
            f"VALUES (0, {aid}, {now}, NULL, NULL, NULL, false, %s, %s, NULL, '0', {parent}, {str(final).lower()}, 0, {aid}, 0)",
            (name, code),
        )

    # ── Icons ────────────────────────────────────────────────────────────────
    # tiles.yuanshen.site serves the real icons (302 → oss.yuanshen.site,
    # which sends Access-Control-Allow-Origin: *). The tag sprite renderer
    # fetches these URLs cross-origin, so the v3 domain (no CORS) breaks it.
    icons = [
        (1, "锚点", "https://tiles.yuanshen.site/icon/Anchor.png"),
        (2, "神像", "https://tiles.yuanshen.site/icon/Statue.png"),
        (3, "秘境", "https://tiles.yuanshen.site/icon/Domain.png"),
        (4, "宝箱", "https://tiles.yuanshen.site/icon/Chest.png"),
        (5, "材料", "https://tiles.yuanshen.site/icon/Item.png"),
    ]
    for iid, name, url in icons:
        cur.execute(
            f'INSERT INTO "genshin_map"."icon" '
            f"(version, id, create_time, update_time, creator_id, updater_id, del_flag, name, url) "
            f"VALUES (0, {iid}, {now}, NULL, NULL, NULL, false, %s, %s)",
            (name, url),
        )

    # ── Item types (the frontend's left panel groups items by type) ─────────
    item_types = [
        (1, "传送锚点", "0", 1),
        (2, "七天神像", "0", 2),
        (3, "秘境", "0", 3),
        (4, "宝箱", "0", 4),
        (5, "材料", "0", 5),
    ]
    for tid, name, icon_tag, sort in item_types:
        cur.execute(
            f'INSERT INTO "genshin_map"."item_type" '
            f"(version, id, create_time, update_time, creator_id, updater_id, del_flag, "
            f"icon_tag, name, content, parent_id, is_final, hidden_flag, sort_index) "
            f"VALUES (0, {tid}, {now}, NULL, NULL, NULL, false, %s, %s, NULL, {tid}, true, 0, {sort})",
            (icon_tag, name),
        )

    # ── Items (area_id points at a child area, e.g. 蒙德城) ────────────────
    items = [
        (1, "传送锚点", 11, "0", 0, 1),
        (2, "七天神像", 11, "0", 0, 2),
        (3, "秘境", 11, "0", 0, 3),
        (4, "珍贵宝箱", 12, "0", 0, 4),
        (5, "普通宝箱", 12, "0", 0, 4),
        (6, "松茸", 13, "0", 0, 5),
        (7, "薄荷", 13, "0", 0, 5),
    ]
    for iid, name, area_id, icon_tag, sort, type_id in items:
        cur.execute(
            f'INSERT INTO "genshin_map"."item" '
            f"(version, id, create_time, update_time, creator_id, updater_id, del_flag, "
            f"name, area_id, default_refresh_time, default_content, default_count, "
            f"icon_tag, icon_style_type, hidden_flag, sort_index, special_flag) "
            f"VALUES (0, {iid}, {now}, NULL, NULL, NULL, false, %s, {area_id}, 0, NULL, 1, %s, 0, 0, {sort}, NULL)",
            (name, icon_tag),
        )
        cur.execute(
            f'INSERT INTO "genshin_map"."item_type_link" '
            f"(version, id, create_time, update_time, creator_id, updater_id, del_flag, "
            f"type_id, item_id) "
            f"VALUES (0, {iid}, {now}, NULL, NULL, NULL, false, {type_id}, {iid})"
        )

    # ── Notices (公告面板) ───────────────────────────────────────────────────
    notices = [
        (1, "欢迎使用本地开发环境", "这是空荧酒馆地图的本地演示环境。底图与点位均为演示数据，登录账号 admin/admin123。"),
        (2, "功能提示", "在左侧选择地区（蒙德）→ 物品类型 → 勾选物品，对应点位会显示在地图上。"),
    ]
    for nid, title, content in notices:
        cur.execute(
            f'INSERT INTO "genshin_map"."notice" '
            f"(version, id, create_time, update_time, creator_id, updater_id, del_flag, "
            f"channel, title, content, valid_time_start, valid_time_end, sort_index) "
            f"VALUES (0, {nid}, {now}, NULL, 1, NULL, false, %s, %s, %s, {now}, NULL, {nid})",
            ('["COMMON"]', title, content),
        )

    # ── Tags (icon-tag sprite; tag_type_link keyed by tag name) ──────────────
    tag_types = [
        (1, "锚点"),
        (2, "神像"),
        (3, "秘境"),
        (4, "宝箱"),
        (5, "材料"),
    ]
    for tid, name in tag_types:
        cur.execute(
            f'INSERT INTO "genshin_map"."tag_type" '
            f"(version, id, create_time, update_time, creator_id, updater_id, del_flag, "
            f"name, parent_id, is_final) "
            f"VALUES (0, {tid}, {now}, NULL, NULL, NULL, false, %s, {tid}, true)",
            (name,),
        )
    tags = [
        (1, "anchor", 1, 1),
        (2, "statue", 1, 2),
        (3, "domain", 1, 3),
        (4, "chest", 1, 4),
        (5, "material", 1, 5),
    ]
    for tid, tag_name, icon_id, type_id in tags:
        cur.execute(
            f'INSERT INTO "genshin_map"."tag" '
            f"(version, id, create_time, update_time, creator_id, updater_id, del_flag, tag, icon_id) "
            f"VALUES (0, {tid}, {now}, NULL, NULL, NULL, false, %s, {icon_id})",
            (tag_name,),
        )
        cur.execute(
            f'INSERT INTO "genshin_map"."tag_type_link" '
            f"(version, id, create_time, update_time, creator_id, updater_id, del_flag, "
            f"type_id, tag_name) "
            f"VALUES (0, {tid}, {now}, NULL, NULL, NULL, false, {type_id}, %s)",
            (tag_name,),
        )

    # ── Markers: ~150 points spread over the Mondstadt region ────────────────
    # Game coordinates: Mondstadt spans roughly x 500..3500, y -4700..-2300
    # (Mondstadt city ≈ [1600, -4050]); the frontend renders markers in this
    # space on top of the 提瓦特 tile map.
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
        x = rng.randint(500, 3500)
        y = -rng.randint(2300, 4700)
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
    cur.execute('SELECT count(*) FROM "genshin_map"."item_type"')
    type_count = cur.fetchone()[0]

    conn.close()
    print(
        f"Demo data seeded: {area_count} areas, {type_count} item types, "
        f"{item_count} items, {marker_count} markers"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
