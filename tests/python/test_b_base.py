# 覆盖面：基础数据域——area / icon_type / icon（urlVariants 合并语义）/
# item_type / item（含 copy、乐观锁）/ item_common 的 CRUD、VO 形态与写权限矩阵。
"""Route B: base data domain regression (area / icon / icon_type / item_type / item / item_common).

All writes are namespaced regress_* under a dedicated regression root area, and
cleaned up at the end (the cleanup itself exercises the recursive area delete).
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from common import auth_ready

if not auth_ready():
    print("skipped: set REGRESSION_PASSWORD")
    sys.exit(0)

from common import login, call, check, check_eq, section, summary

admin, _ = login("regress_admin")
manager, _ = login("regress_manager")
punctuate, _ = login("regress_punctuate")
user_t, _ = login("regress_user")

CREATED = {"area_ids": [], "icon_ids": [], "icon_type_ids": [], "item_type_ids": [], "item_ids": []}


def wrap_ok(pr):
    return isinstance(pr, dict) and pr.get("error") is False


section("B-1 area read paths")
st, pr = call("POST", "/api/area/get/list", token=admin, body={})
check("area list (empty body) 200", st == 200, f"st={st} {str(pr)[:150]}")
if wrap_ok(pr) and isinstance(pr.get("data"), list):
    check("area list returns data array", True)
    # 依赖库内已有数据（dev 库非空；全新本地库可能为空）。
    check("area list non-empty (pre-seeded data)", len(pr["data"]) > 0, f"len={len(pr['data'])}")
    first = pr["data"][0]
    for k in ("id", "name", "parentId", "hiddenFlag", "isFinal", "version"):
        check(f"area VO has {k}", k in first, f"keys={list(first.keys())[:15]}")
    root_id = first["id"]
else:
    root_id = None
    check("area list wrapped in R", False, str(pr)[:200])

# single get on an existing area
if root_id:
    st, pr = call("POST", f"/api/area/get/{root_id}", token=admin)
    check("area get existing 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:150]}")

st, pr = call("POST", "/api/area/get/999999999", token=admin)
check("area get nonexistent -> error in R", st == 200 and isinstance(pr, dict) and pr.get("error") is True, f"st={st} {str(pr)[:150]}")

# role visibility
st_vis, pr_vis = call("POST", "/api/area/get/list", token=user_t, body={})
check("MAP_USER can list areas", st_vis == 200 and wrap_ok(pr_vis), f"st={st_vis}")

section("B-1 area create / update / delete (own namespace)")
st, pr = call("PUT", "/api/area/add", token=manager, body={
    "name": "regress_root_area", "code": "RGX", "content": "regression root",
    "iconId": 0, "parentId": -1, "isFinal": False, "hiddenFlag": 0, "sortIndex": 9999,
})
check("area add (manager) 200", st == 200, f"st={st} {str(pr)[:200]}")
reg_root = pr.get("data") if wrap_ok(pr) else None
check("area add returns new id", isinstance(reg_root, int), str(pr)[:150])
CREATED["area_ids"].append(reg_root)

st, pr = call("POST", f"/api/area/get/{reg_root}", token=manager)
check("created area readable", wrap_ok(pr) and pr["data"]["name"] == "regress_root_area", str(pr)[:150])
ver = pr["data"]["version"] if wrap_ok(pr) else 0

# update: rename via full request
st, pr = call("POST", "/api/area/update", token=manager, body={
    "id": reg_root, "version": ver,
    "name": "regress_root_area_v2", "code": "RGX", "content": None, "iconId": 0,
    "parentId": -1, "isFinal": False, "hiddenFlag": 0, "sortIndex": 9998,
})
check("area update 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:200]}")

st, pr = call("POST", f"/api/area/get/{reg_root}", token=manager)
check("area update persisted", wrap_ok(pr) and pr["data"]["name"] == "regress_root_area_v2", str(pr)[:150])

# stale version -> optimistic lock rejection
st, pr = call("POST", "/api/area/update", token=manager, body={
    "id": reg_root, "version": ver,
    "name": "regress_root_area_v3", "code": "RGX", "content": None, "iconId": 0,
    "parentId": -1, "isFinal": False, "hiddenFlag": 0, "sortIndex": 9997,
})
print(f"  info: stale-version update -> st={st} err={str(pr)[:120]}")

# sub area under regression root (final)
st, pr = call("PUT", "/api/area/add", token=manager, body={
    "name": "regress_sub_area", "code": "RGXS", "content": None, "iconId": 0,
    "parentId": reg_root, "isFinal": True, "hiddenFlag": 0, "sortIndex": 1,
})
check("sub area add 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:150]}")
reg_sub = pr.get("data") if wrap_ok(pr) else None
CREATED["area_ids"].append(reg_sub)

# permission: MAP_USER cannot add area
st, pr = call("PUT", "/api/area/add", token=user_t, body={
    "name": "regress_forbidden", "parentId": -1, "isFinal": True, "iconId": 0,
})
check("MAP_USER area add forbidden (403)", st == 403, f"st={st}")

section("B-2 icon_type + icon")
st, pr = call("PUT", "/api/icon_type/add", token=manager, body={
    "name": "regress_itype", "parentId": -1, "content": None, "iconId": 0,
    "hiddenFlag": 0, "isFinal": True, "sortIndex": 999,
})
check("icon_type add 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:200]}")
itype = pr.get("data") if wrap_ok(pr) else None
CREATED["icon_type_ids"].append(itype)

st, pr = call("POST", "/api/icon_type/get/list", token=manager, body={})
check("icon_type list 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:100]}")

# icon add with urlVariants custom keys
st, pr = call("PUT", "/api/icon/add", token=manager, body={
    "typeIdList": [itype], "name": "regress_icon_a", "tag": "regress_tag_a",
    "urlVariants": {"hd": "https://example.com/a_hd.png", "mobile": "https://example.com/a_m.png", "custom_key": "https://example.com/a_ck.png"},
})
check("icon add 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:200]}")
icon_a = pr.get("data") if wrap_ok(pr) else None
CREATED["icon_ids"].append(icon_a)

st, pr = call("PUT", "/api/icon/add", token=manager, body={
    "typeIdList": [itype], "name": "regress_icon_b", "tag": "regress_tag_a",
    "urlVariants": {"hd": "https://example.com/b_hd.png"},
})
check("duplicate tag rejected", isinstance(pr, dict) and pr.get("error") is True, f"st={st} {str(pr)[:150]}")

st, pr = call("GET", f"/api/icon/get/single/{icon_a}", token=manager)
check("icon get single 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:150]}")
if wrap_ok(pr) and isinstance(pr.get("data"), dict):
    uv = pr["data"].get("urlVariants") or {}
    check("urlVariants roundtrip hd", uv.get("hd") == "https://example.com/a_hd.png", str(uv)[:120])
    check("urlVariants roundtrip custom key", uv.get("custom_key") == "https://example.com/a_ck.png", str(uv)[:120])

# icon update: Map-merge semantics — key-level patch
st, pr = call("GET", f"/api/icon/get/single/{icon_a}", token=manager)
ver = pr["data"]["version"] if wrap_ok(pr) and isinstance(pr.get("data"), dict) else 0
st, pr = call("POST", "/api/icon/update", token=manager, body={
    "id": icon_a, "version": ver, "typeIdList": [itype], "name": "regress_icon_a",
    "tag": "regress_tag_a", "urlVariants": {"mobile": None, "new_key": "https://example.com/new.png"},
})
check("icon update 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:200]}")
st, pr = call("GET", f"/api/icon/get/single/{icon_a}", token=manager)
if wrap_ok(pr) and isinstance(pr.get("data"), dict):
    uv = pr["data"].get("urlVariants") or {}
    check("urlVariants null deletes key", "mobile" not in uv, str(uv)[:150])
    check("urlVariants new key added", uv.get("new_key") == "https://example.com/new.png", str(uv)[:150])
    check("urlVariants untouched key preserved", uv.get("hd") == "https://example.com/a_hd.png", str(uv)[:150])

section("B-3 item_type + item")
st, pr = call("PUT", "/api/item_type/add", token=manager, body={
    "name": "regress_itemtype_root", "parentId": -1, "content": None, "iconId": 0,
    "hiddenFlag": 0, "isFinal": False, "sortIndex": 999,
})
check("item_type add 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:150]}")
it_root = pr.get("data") if wrap_ok(pr) else None
CREATED["item_type_ids"].append(it_root)

st, pr = call("PUT", "/api/item_type/add", token=manager, body={
    "name": "regress_itemtype_child", "parentId": it_root, "content": None, "iconId": 0,
    "hiddenFlag": 0, "isFinal": True, "sortIndex": 1,
})
check("item_type child add 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:150]}")
it_child = pr.get("data") if wrap_ok(pr) else None
CREATED["item_type_ids"].append(it_child)

# add with self as parent
st, pr = call("PUT", "/api/item_type/add", token=manager, body={
    "name": "regress_itemtype_self", "parentId": 999999999, "content": None, "iconId": 0,
    "hiddenFlag": 0, "isFinal": True, "sortIndex": 1, "id": 999999999,
})
print(f"  info: item_type add parentId==id -> st={st} {str(pr)[:120]}")

st, pr = call("POST", "/api/item_type/get/list", token=manager, body={"parentIdList": [it_root]})
check("item_type list by parent 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:150]}")

# item add in regression sub area
st, pr = call("PUT", "/api/item/add", token=manager, body={
    "areaId": reg_sub, "defaultContent": "rc", "defaultCount": 1, "defaultRefreshTime": 123,
    "hiddenFlag": 0, "iconStyleType": 0, "iconId": icon_a,
    "name": "regress_item_a", "priority": 0, "refreshTime": 456, "specialFlag": 10,
    "typeIdList": [it_child],
})
check("item add 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:200]}")
item_a = pr.get("data") if wrap_ok(pr) else None
CREATED["item_ids"].append(item_a)

# item with nonexistent type id -> error
st, pr = call("PUT", "/api/item/add", token=manager, body={
    "areaId": reg_sub, "iconId": icon_a, "name": "regress_item_bad_type",
    "typeIdList": [999999999], "iconStyleType": 0,
})
check("item add bad type id -> R error", isinstance(pr, dict) and pr.get("error") is True, f"st={st} {str(pr)[:150]}")

# item list by area
st, pr = call("POST", "/api/item/get/list", token=manager, body={"areaIdList": [reg_sub]})
check("item list by area 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:150]}")
if wrap_ok(pr) and isinstance(pr.get("data"), list):
    names = [i.get("name") for i in pr["data"]]
    check("created item visible in list", "regress_item_a" in names, str(names)[:150])
    mine = next((i for i in pr["data"] if i.get("name") == "regress_item_a"), None)
    if mine:
        check("item specialFlag roundtrip", mine.get("specialFlag") == 10, str(mine.get("specialFlag")))
        check("item refreshTime roundtrip", mine.get("refreshTime") == 456, str(mine.get("refreshTime")))
        check("item countSplit present", "countSplit" in mine, str(list(mine.keys())))

# item update editSame=0 (self only)
st, pr = call("POST", "/api/item/get/list", token=manager, body={"areaIdList": [reg_sub]})
mine = next((i for i in pr["data"] if i.get("name") == "regress_item_a"), None)
ver = mine["version"]
st, pr = call("POST", "/api/item/update/0", token=manager, body={
    "id": item_a, "version": ver, "areaId": reg_sub, "name": "regress_item_a_v2",
    "iconId": icon_a, "typeIdList": [it_child], "iconStyleType": 0,
    "defaultContent": "rc2", "defaultCount": 2, "defaultRefreshTime": 111,
    "hiddenFlag": 0, "priority": 0, "refreshTime": 789, "specialFlag": 10,
})
check("item update/0 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:200]}")
st, pr = call("POST", "/api/item/get/list", token=manager, body={"areaIdList": [reg_sub]})
names = [i.get("name") for i in (pr.get("data") or [])]
check("item renamed after update/0", "regress_item_a_v2" in names, str(names)[:150])

# item copy into another regression area
st, pr = call("PUT", "/api/area/add", token=manager, body={
    "name": "regress_sub_area2", "parentId": reg_root, "isFinal": True, "iconId": 0, "hiddenFlag": 0,
})
reg_sub2 = pr.get("data") if wrap_ok(pr) else None
CREATED["area_ids"].append(reg_sub2)
st, pr = call("PUT", f"/api/item/copy/{reg_sub2}", token=manager, body=[item_a])
check("item copy 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:200]}")
st, pr = call("POST", "/api/item/get/list", token=manager, body={"areaIdList": [reg_sub2]})
names = [i.get("name") for i in (pr.get("data") or [])]
check("copied item exists in target area", "regress_item_a_v2" in names, str(names)[:150])

section("B item_common")
st, pr = call("PUT", "/api/item_common/add", token=manager, body={
    "defaultContent": "rcc", "defaultCount": 1, "defaultRefreshTime": 1,
    "hiddenFlag": 0, "iconStyleType": 0, "iconId": icon_a, "name": "regress_common_item",
    "priority": 0, "refreshTime": 1, "specialFlag": 0,
})
check("item_common add 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:200]}")
common_id = pr.get("data") if wrap_ok(pr) else None

st, pr = call("POST", "/api/item_common/get/list_all", token=manager, body={})
check("item_common list_all 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:120]}")

section("B permission matrix (writes)")
st, _ = call("PUT", "/api/item/add", token=user_t, body={"areaId": reg_sub, "name": "x", "typeIdList": [it_child], "iconId": 0})
check("MAP_USER item add forbidden", st == 403, f"st={st}")
st, _ = call("DELETE", f"/api/icon/delete/{icon_a}", token=user_t)
check("MAP_USER icon delete forbidden", st == 403, f"st={st}")
st, _ = call("PUT", "/api/area/add", token=punctuate, body={"name": "x", "parentId": -1, "isFinal": True, "iconId": 0})
check("MAP_PUNCTUATE area add forbidden", st == 403, f"st={st}")

# ============ cleanup ============
section("cleanup")
st, pr = call("DELETE", f"/api/item_common/delete", token=admin, body=[common_id])
print(f"  item_common delete: {st} {str(pr)[:80]}")
st, pr = call("DELETE", f"/api/item/delete/{item_a}", token=admin)
print(f"  item delete: {st}")
st, pr = call("DELETE", f"/api/item_type/delete/{it_child}", token=admin)
print(f"  item_type child delete: {st}")
st, pr = call("DELETE", f"/api/item_type/delete/{it_root}", token=admin)
print(f"  item_type root delete: {st}")
st, pr = call("DELETE", f"/api/icon/delete/{icon_a}", token=admin)
print(f"  icon delete: {st}")
st, pr = call("DELETE", f"/api/icon_type/delete/{itype}", token=admin)
print(f"  icon_type delete: {st}")
# recursive area delete: removes sub areas + their items/markers
st, pr = call("DELETE", f"/api/area/{reg_root}", token=admin)
check("recursive area delete 200", st == 200, f"st={st} {str(pr)[:120]}")
st, pr = call("POST", f"/api/area/get/{reg_root}", token=admin)
check("area gone after delete", isinstance(pr, dict) and pr.get("error") is True, f"st={st} {str(pr)[:120]}")
st, pr = call("POST", "/api/item/get/list", token=admin, body={"areaIdList": [reg_sub]})
check("items in deleted area gone", wrap_ok(pr) and not pr.get("data"), f"st={st} {str(pr)[:150]}")

sys.exit(summary())
