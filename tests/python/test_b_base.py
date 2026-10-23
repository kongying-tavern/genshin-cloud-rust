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


def paged_records(pr):
    """分页封套 {record,total,size} 取记录数组（Java PageListVo 契约）。"""
    data = pr.get("data") if isinstance(pr, dict) else None
    if isinstance(data, dict):
        rec = data.get("record")
        return rec if isinstance(rec, list) else []
    return data if isinstance(data, list) else []

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
    "name": "regress_forbidden",
    "code": "REGRESS_FORB", "parentId": -1, "isFinal": True, "iconId": 0,
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
    "typeIdList": [itype], "tag": "regress_tag_a",
    "urlVariants": {"hd": "https://example.com/a_hd.png", "mobile": "https://example.com/a_m.png", "custom_key": "https://example.com/a_ck.png"},
})
check("icon add 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:200]}")
icon_a = pr.get("data") if wrap_ok(pr) else None
CREATED["icon_ids"].append(icon_a)

st, pr = call("PUT", "/api/icon/add", token=manager, body={
    "typeIdList": [itype], "tag": "regress_tag_a",
    "urlVariants": {"hd": "https://example.com/b_hd.png"},
})
check("duplicate tag rejected", isinstance(pr, dict) and pr.get("error") is True, f"st={st} {str(pr)[:150]}")

st, pr = call("POST", f"/api/icon/get/single/{icon_a}", token=manager)
check("icon get single 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:150]}")
if wrap_ok(pr) and isinstance(pr.get("data"), dict):
    uv = pr["data"].get("urlVariants") or {}
    check("urlVariants roundtrip hd", uv.get("hd") == "https://example.com/a_hd.png", str(uv)[:120])
    check("urlVariants roundtrip custom key", uv.get("custom_key") == "https://example.com/a_ck.png", str(uv)[:120])
    vo = pr["data"]
    # Java IconVo 全字段契约：tag（非 name）/审计字段/typeIdList/description
    check("icon VO uses tag field", vo.get("tag") == "regress_tag_a", str(vo)[:150])
    check("icon VO carries creatorId", isinstance(vo.get("creatorId"), int), str(vo)[:150])
    check("icon VO carries typeIdList", itype in (vo.get("typeIdList") or []), str(vo)[:150])
    check("icon VO carries createTime", isinstance(vo.get("createTime"), (int, float)), str(vo)[:150])
    check("icon VO carries description", "description" in vo, str(vo)[:150])

# icon list：tag 搜索 + 信封 users + PageListVo {record,total,size}
st, pr = call("POST", "/api/icon/get/list", token=manager, body={"tag": "regress_tag_a", "current": 1, "size": 10})
check("icon list by tag 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:200]}")
if wrap_ok(pr) and isinstance(pr.get("data"), dict):
    d = pr["data"]
    check("icon list envelope has record/total/size",
          all(k in d for k in ("record", "total", "size")), str(list(d.keys())))
    check("icon list finds regress icon", any(r.get("id") == icon_a for r in d.get("record") or []), str(d)[:150])
    check("icon list envelope users map populated",
          isinstance(pr.get("users"), dict) and len(pr.get("users") or {}) > 0, str(pr.get("users"))[:120])

# icon update: Map-merge semantics — key-level patch
st, pr = call("POST", f"/api/icon/get/single/{icon_a}", token=manager)
ver = pr["data"]["version"] if wrap_ok(pr) and isinstance(pr.get("data"), dict) else 0
st, pr = call("POST", "/api/icon/update", token=manager, body={
    "id": icon_a, "version": ver, "typeIdList": [itype], "tag": "regress_tag_a", "urlVariants": {"mobile": None, "new_key": "https://example.com/new.png"},
})
check("icon update 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:200]}")
st, pr = call("POST", f"/api/icon/get/single/{icon_a}", token=manager)
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

st, pr = call("POST", "/api/item_type/get/list_all", token=manager, body={})
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
item_rows = paged_records(pr)
if item_rows:
    names = [i.get("name") for i in item_rows]
    check("created item visible in list", "regress_item_a" in names, str(names)[:150])
    mine = next((i for i in item_rows if i.get("name") == "regress_item_a"), None)
    if mine:
        check("item specialFlag roundtrip", mine.get("specialFlag") == 10, str(mine.get("specialFlag")))
        check("item defaultRefreshTime roundtrip", mine.get("defaultRefreshTime") == 123, str(mine.get("defaultRefreshTime")))
        check("item countSplit present", "countSplit" in mine, str(list(mine.keys())))

# item update editSame=0 (self only)
st, pr = call("POST", "/api/item/get/list", token=manager, body={"areaIdList": [reg_sub]})
rows = paged_records(pr)
check("item list by area returns records", isinstance(rows, list) and bool(rows), str(pr)[:160])
mine = next((i for i in (rows or []) if isinstance(i, dict) and i.get("name") == "regress_item_a"), None)
ver = (mine or {}).get("version", 0)
# Java updateItem：body 为 ItemVo 数组（批量）
st, pr = call("POST", "/api/item/update/0", token=manager, body=[{
    "id": item_a, "version": ver, "areaId": reg_sub, "name": "regress_item_a_v2",
    "iconId": icon_a, "typeIdList": [it_child], "iconStyleType": 0,
    "defaultContent": "rc2", "defaultCount": 2, "defaultRefreshTime": 111,
    "hiddenFlag": 0, "priority": 0, "refreshTime": 789, "specialFlag": 10,
}])
check("item update/0 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:200]}")
st, pr = call("POST", "/api/item/get/list", token=manager, body={"areaIdList": [reg_sub]})
names = [i.get("name") for i in paged_records(pr)]
check("item renamed after update/0", "regress_item_a_v2" in names, str(names)[:150])

# item copy into another regression area
st, pr = call("PUT", "/api/area/add", token=manager, body={
    "name": "regress_sub_area2", "code": "RGXS2", "parentId": reg_root, "isFinal": True,
    "iconId": 0, "hiddenFlag": 0,
})
reg_sub2 = pr.get("data") if wrap_ok(pr) else None
CREATED["area_ids"].append(reg_sub2)
st, pr = call("PUT", f"/api/item/copy/{reg_sub2}", token=manager, body=[item_a])
check("item copy 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:200]}")
st, pr = call("POST", "/api/item/get/list", token=manager, body={"areaIdList": [reg_sub2]})
names = [i.get("name") for i in paged_records(pr)]
check("copied item exists in target area", "regress_item_a_v2" in names, str(names)[:150])

section("B item_common")
# Java ItemCommonController.add：body 为 itemId 数组（List<Long>）
st, pr = call("PUT", "/api/item_common/add", token=manager, body=[item_a])
check("item_common add 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:200]}")
common_id = pr.get("data") if wrap_ok(pr) else None

st, pr = call("POST", "/api/item_common/get/list", token=manager, body={"current": 1, "size": 10})
check("item_common list 200", st == 200 and wrap_ok(pr), f"st={st} {str(pr)[:120]}")

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
check("items in deleted area gone", wrap_ok(pr) and not paged_records(pr), f"st={st} {str(pr)[:150]}")

sys.exit(summary())
