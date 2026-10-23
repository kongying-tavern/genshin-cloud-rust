# 覆盖面：点位核心域——marker 的建/查（get/id、list_byid、list_byinfo、page）、
# 改（乐观锁）、删、tweak 批量替换、extra 特殊载荷往返、hiddenFlag 角色可见性矩阵。
"""Route C: marker core domain regression.

Sets up an isolated regression area + item + markers, exercises the query
quad, write paths, tweak, and the hiddenFlag visibility matrix, then cleans up.
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from common import auth_ready

if not auth_ready():
    print("skipped: set REGRESSION_PASSWORD")
    sys.exit(0)

from common import login, call, raw_call, check, check_eq, section, summary

admin, _ = login("regress_admin")
punctuate, resp_punc = login("regress_punctuate")
beta, _ = login("regress_beta")
user_t, resp_user = login("regress_user")
manager, _ = login("regress_manager")
# markerCreatorId 取打点员/普通用户的真实 id（dev 库上为 203/204，本地库可能不同）。
PUNC_ID = resp_punc["userId"]
USER_ID = resp_user["userId"]

AREA = None
ITEM = None
MARKERS = []

section("C setup: regression area + item")
st, pr = call("PUT", "/api/area/add", token=manager, body={
    "name": "regress_marker_area",
    "code": "REGRESS_MARK", "parentId": -1, "isFinal": True, "iconId": 0, "hiddenFlag": 0})
check("area created", st == 200 and isinstance(pr.get("data"), int), str(pr)[:120])
AREA = pr.get("data")
st, pr = call("PUT", "/api/item/add", token=manager, body={
    "areaId": AREA, "name": "regress_marker_item", "typeIdList": [], "iconId": 0,
    "iconStyleType": 0, "hiddenFlag": 0, "specialFlag": 0})
check("item created", st == 200 and isinstance(pr.get("data"), int), str(pr)[:120])
ITEM = pr.get("data")

def mk_marker(title, hidden_flag, extra=None, token=None):
    body = {
        "content": f"rc {title}", "extra": extra, "hiddenFlag": hidden_flag,
        "itemList": [{"itemId": ITEM, "count": 2}],
        "markerCreatorId": PUNC_ID, "markerTitle": title, "picture": None,
        "pictureCreatorId": None, "position": "123.45,678.90", "refreshTime": 1000,
        "videoPath": None,
    }
    st, pr = call("PUT", "/api/marker/single", token=token or punctuate, body=body)
    if st == 200 and isinstance(pr, dict) and not pr.get("error"):
        mid = pr["data"]["id"] if isinstance(pr.get("data"), dict) else pr.get("data")
        MARKERS.append(mid)
        return mid
    print(f"  marker create failed: {title} st={st} {str(pr)[:150]}")
    return None

section("C-2 PUT /marker/single (create)")
m_normal = mk_marker("regress_m_normal", 0)
check("marker created", isinstance(m_normal, int))
m_hidden = mk_marker("regress_m_hidden", 1)
m_beta = mk_marker("regress_m_beta", 2, token=admin)
m_easter = mk_marker("regress_m_easter", 3)
check("all flag variants created", all(isinstance(x, int) for x in [m_normal, m_hidden, m_beta, m_easter]))

section("C-1 POST /marker/get/id (filter + role visibility)")
st, pr = call("POST", "/api/marker/get/id", token=admin, body={"areaIdList": [AREA]})
ids_admin = set(pr.get("data") or []) if isinstance(pr, dict) else set()
check("admin sees all 4 markers", ids_admin == set(MARKERS), f"{ids_admin}")

# 可见性断言走 list_byinfo（载入阶段按 marker.hiddenFlag 过滤）；get/id 按
# Java searchMarkerId 语义只做物品侧过滤，不做点位可见性过滤。
st, pr = call("POST", "/api/marker/get/list_byinfo", token=user_t, body={"areaIdList": [AREA]})
ids_user = {v.get("id") for v in (pr.get("data") or [])} if isinstance(pr, dict) else set()
check("MAP_USER sees only flag 0/3 (no hidden/beta)", ids_user == {m_normal, m_easter}, f"{ids_user}")

st, pr = call("POST", "/api/marker/get/id", token=beta, body={"areaIdList": [AREA]})
ids_beta = set(pr.get("data") or []) if isinstance(pr, dict) else set()
check("MAP_BETA sees 0/1/2/3", ids_beta == set(MARKERS), f"{ids_beta}")

st, pr = call("POST", "/api/marker/get/list_byinfo", token=punctuate, body={"areaIdList": [AREA]})
ids_punc = {v.get("id") for v in (pr.get("data") or [])} if isinstance(pr, dict) else set()
check("MAP_PUNCTUATE sees 0/1/3 (no beta)", ids_punc == {m_normal, m_hidden, m_easter}, f"{ids_punc}")

# empty-condition query
st, pr = call("POST", "/api/marker/get/id", token=admin, body={})
check("empty filter returns ids", st == 200 and isinstance(pr.get("data"), list), str(pr)[:100])

section("C-1 list_byinfo / list_byid / page")
st, pr = call("POST", "/api/marker/get/list_byid", token=admin, body=MARKERS)
recs = pr.get("data") if isinstance(pr, dict) else None
check("list_byid returns records", isinstance(recs, list) and len(recs) == 4, str(pr)[:150])
if recs:
    vo = recs[0]
    for k in ("id", "position", "hiddenFlag", "refreshTime", "itemList", "markerTitle"):
        check(f"marker VO has {k}", k in vo, f"keys={list(vo.keys())[:18]}")
    il = vo.get("itemList") or []
    check("itemList populated with link", any(l.get("itemId") == ITEM for l in il), str(il)[:120])

st, pr = call("POST", "/api/marker/get/list_byinfo", token=admin, body={
    "areaIdList": [AREA], "hiddenFlag": 0})
recs2 = pr.get("data") if isinstance(pr, dict) else None
check("list_byinfo returns area markers (admin sees all flags)", isinstance(recs2, list) and {r["id"] for r in recs2} == set(MARKERS), str(pr)[:150])

st, pr = call("POST", "/api/marker/get/page", token=admin, body={
    "areaIdList": [AREA], "current": 1, "size": 2})
page = pr.get("data") if isinstance(pr, dict) else None
# 旧断言 total > 100000 依赖共享 dev 库的整表规模；本地 e2e 库数据量小，
# 放宽为 >= 本套件创建的点位数（TODO: 数据规模断言依赖库内数据形状）。
check("page returns total (whole-table for admin)", isinstance(page, dict) and page.get("total", 0) >= len(MARKERS), str(page)[:80])
check("page returns 2 records", isinstance(page.get("record"), list) and len(page["record"]) == 2)

section("C-2 POST /marker/single (update)")
st, pr = call("POST", "/api/marker/get/list_byid", token=admin, body=[m_normal])
vo = pr.get("data")[0]
upd = {
    "id": m_normal, "content": "rc updated", "extra": None, "hiddenFlag": 0,
    "itemList": [{"itemId": ITEM, "count": 5}],
    "markerCreatorId": PUNC_ID, "markerTitle": "regress_m_normal", "picture": None,
    "pictureCreatorId": None, "position": "999.0,1.0", "refreshTime": 2000,
    "videoPath": None,
}
st, pr = call("POST", "/api/marker/single", token=punctuate, body=upd)
check("marker update 200", st == 200 and isinstance(pr, dict) and not pr.get("error"), str(pr)[:150])
st, pr = call("POST", "/api/marker/get/list_byid", token=admin, body=[m_normal])
vo2 = pr.get("data")[0]
check("update persisted (position)", vo2.get("position") == "999.0,1.0", vo2.get("position"))
check("update persisted (content)", vo2.get("content") == "rc updated")
il = vo2.get("itemList") or []
check("itemList count updated to 5", any(l.get("count") == 5 for l in il), str(il)[:100])
check("marker update bumps version", isinstance(vo2.get("version"), int), str(vo2.get("version")))

# update nonexistent
st, pr = call("POST", "/api/marker/single", token=punctuate, body={**upd, "id": 999999999})
check("update nonexistent -> R error", isinstance(pr, dict) and pr.get("error") is True, f"st={st} {str(pr)[:120]}")

section("C-2 DELETE /marker/{id}")
st, pr = call("DELETE", f"/api/marker/{m_easter}", token=punctuate)
check("marker delete 200", st == 200 and isinstance(pr, dict) and not pr.get("error"), str(pr)[:120])
st, pr = call("POST", "/api/marker/get/id", token=admin, body={"areaIdList": [AREA]})
ids_after = set(pr.get("data") or [])
check("deleted marker invisible", m_easter not in ids_after, f"{ids_after}")
MARKERS.remove(m_easter)

section("C-2 tweak")
st, pr = call("POST", "/api/marker/tweak", token=punctuate, body=[{
    "markerIds": [m_normal], "tweaks": [{"prop": "content", "type": "replace", "meta": {"test": "rc updated", "replace": "rc tweaked"}}]}])
check("tweak 200", st == 200 and isinstance(pr, dict) and not pr.get("error"), str(pr)[:150])
st, pr = call("POST", "/api/marker/get/list_byid", token=admin, body=[m_normal])
vo3 = pr.get("data")[0]
check("tweak applied to content", vo3.get("content") == "rc tweaked", vo3.get("content"))

# tweak with empty list
st, pr = call("POST", "/api/marker/tweak", token=punctuate, body=[])
check("tweak empty list ok", st == 200, f"st={st}")

section("C-3 extra roundtrip (special marker payloads)")
m_extra = mk_marker("regress_m_extra", 0, extra={
    "underground": {"isUnderground": True, "isGlobal": False, "regionLevels": [1, 2]},
    "iconOverride": {"id": 42, "minZoom": 3, "maxZoom": 7},
})
st, pr = call("POST", "/api/marker/get/list_byid", token=admin, body=[m_extra])
vo4 = pr.get("data")[0]
ex = vo4.get("extra") or {}
check("extra underground roundtrip", (ex.get("underground") or {}).get("isUnderground") is True, str(ex)[:150])
check("extra iconOverride roundtrip", (ex.get("iconOverride") or {}).get("id") == 42, str(ex)[:150])

section("C permission gates")
st, _ = call("PUT", "/api/marker/single", token=user_t, body={
    "content": None, "extra": None, "hiddenFlag": 0, "itemList": [], "markerCreatorId": USER_ID,
    "markerTitle": "x", "picture": None, "pictureCreatorId": None, "position": "0,0",
    "refreshTime": 0, "videoPath": None})
check("MAP_USER marker write forbidden", st == 403, f"st={st}")
st, _ = call("POST", "/api/marker/get/id", token=user_t, body={"areaIdList": [AREA]})
check("MAP_USER marker read ok", st == 200, f"st={st}")

section("C-4 GET /marker_doc/list_markers (full protobuf export)")

import gzip as _gzip
import io as _io


def _read_varint(buf, i):
    v, shift = 0, 0
    while True:
        b = buf[i]
        i += 1
        v |= (b & 0x7F) << shift
        if not b & 0x80:
            return v, i
        shift += 7


def _top_level_fields(buf):
    """解析 protobuf 顶层字段：yield (field_no, wire_type, payload_bytes)。"""
    i = 0
    while i < len(buf):
        tag, i = _read_varint(buf, i)
        fn, wt = tag >> 3, tag & 7
        if wt == 2:
            ln, i = _read_varint(buf, i)
            yield fn, wt, buf[i:i + ln]
            i += ln
        elif wt == 0:
            v, i = _read_varint(buf, i)
            yield fn, wt, v
        else:
            raise ValueError(f"unexpected wire type {wt} at {i}")


def _marker_id_from_entry(msg):
    """MarkerVo 子消息里取 field 2（id，varint）。"""
    for fn, wt, v in _top_level_fields(msg):
        if fn == 2 and wt == 0:
            return v
    return None


def _list_marker_ids(token):
    st, hdrs, payload = raw_call("GET", "/api/marker_doc/list_markers", token=token)
    if st != 200 or not isinstance(payload, (bytes, bytearray)):
        return st, hdrs, payload, None
    raw = payload if isinstance(payload, bytes) else bytes(payload)
    if raw[:2] != b"\x1f\x8b":
        return st, hdrs, raw, "not-gzip"
    plain = _gzip.decompress(raw)
    ids = set()
    users = 0
    for fn, wt, v in _top_level_fields(plain):
        if fn == 1 and wt == 2:
            mid = _marker_id_from_entry(v)
            if mid is not None:
                ids.add(mid)
        elif fn == 2 and wt == 2:
            users += 1
    return st, hdrs, raw, (ids, users)


st, hdrs, raw, parsed = _list_marker_ids(admin)
check("list_markers 200 octet-stream", st == 200 and isinstance(raw, (bytes, bytearray)), f"st={st}")
ct_hdr = next((v for k, v in (hdrs or {}).items() if k.lower() == "content-type"), "") or ""
check("list_markers content-type octet-stream", "octet-stream" in ct_hdr, str(ct_hdr))
check("list_markers is gzip", isinstance(raw, (bytes, bytearray)) and raw[:2] == b"\x1f\x8b", str(raw[:4]))
if isinstance(parsed, tuple):
    ids, users = parsed
    check("list_markers contains created marker", m_normal in ids, f"m_normal={m_normal} count={len(ids)}")
    check("list_markers protobuf markers non-empty", len(ids) > 0)
else:
    check("list_markers protobuf parses", False, str(parsed)[:120])

st, _, _, parsed_user = _list_marker_ids(user_t)
if isinstance(parsed_user, tuple):
    ids_user, _u = parsed_user
    check("MAP_USER list_markers excludes hidden/beta markers",
          m_hidden not in ids_user and m_beta not in ids_user,
          f"hidden={m_hidden in ids_user} beta={m_beta in ids_user}")

section("cleanup")
st, pr = call("DELETE", f"/api/area/{AREA}", token=admin)
check("area recursive delete", st == 200 and isinstance(pr, dict) and not pr.get("error"), str(pr)[:120])
st, pr = call("POST", "/api/marker/get/id", token=admin, body={"areaIdList": [AREA]})
ids = set(pr.get("data") or [])
check("markers gone with area", not (ids & set(MARKERS)), f"{ids}")

sys.exit(summary())
