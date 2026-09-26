# 覆盖面：点位联动域——link 建组、跨组合并、get/list 与 get/graph 一致性、
# 按组删除、自环拒绝、读写权限门。
"""Route D: marker_link domain regression (isolated namespace, cleanup at end)."""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from common import auth_ready

if not auth_ready():
    print("skipped: set REGRESSION_PASSWORD")
    sys.exit(0)

from common import login, call, check, check_eq, section, summary

punctuate, resp_punc = login("regress_punctuate")
admin, _ = login("regress_admin")
manager, _ = login("regress_manager")
user_t, _ = login("regress_user")
PUNC_ID = resp_punc["userId"]  # markerCreatorId 用打点员真实 id（动态推导）

# setup: own area + item + 4 markers
st, pr = call("PUT", "/api/area/add", token=manager, body={
    "name": "regress_link_area", "parentId": -1, "isFinal": True, "iconId": 0, "hiddenFlag": 0})
AREA = pr["data"]
st, pr = call("PUT", "/api/item/add", token=manager, body={
    "areaId": AREA, "name": "regress_link_item", "typeIdList": [], "iconId": 0,
    "iconStyleType": 0, "hiddenFlag": 0})
ITEM = pr["data"]

M = []
for i in range(4):
    st, pr = call("PUT", "/api/marker/single", token=punctuate, body={
        "content": f"rd m{i}", "extra": None, "hiddenFlag": 0,
        "itemList": [{"itemId": ITEM, "count": 1}], "markerCreatorId": PUNC_ID,
        "markerTitle": f"regress_link_m{i}", "picture": None, "pictureCreatorId": None,
        "position": f"{i}.0,{i}.5", "refreshTime": 0, "videoPath": None})
    M.append(pr["data"]["id"] if isinstance(pr.get("data"), dict) else pr["data"])

section("D-1 link creates group")
st, pr = call("POST", "/api/marker_link/link", token=punctuate, body=[
    {"fromId": M[0], "toId": M[1]}])
check("link 200", st == 200 and isinstance(pr, dict) and not pr.get("error"), str(pr)[:150])
grp1 = pr.get("data")
check("link returns groupId", isinstance(grp1, str) and grp1, str(pr)[:150])
st, pr = call("POST", "/api/marker_link/get/list", token=punctuate, body={"groupIds": [grp1]})
links = pr.get("data") if isinstance(pr, dict) else None
# Java contract: Map<markerId, List<MarkerLinkageVo>>
if isinstance(links, dict):
    flat = [l for lst in links.values() for l in lst]
    check("get/list keyed by groupId", set(links.keys()) == {grp1}, str(list(links.keys()))[:120])
else:
    flat = links or []
check("link edge visible in get/list", any(l.get("fromId") == M[0] and l.get("toId") == M[1] for l in flat), str(flat)[:200])

# both markers expose the same linkageId
st, pr = call("POST", "/api/marker/get/list_byid", token=admin, body=M)
recs = pr.get("data") or []
by_id = {r["id"]: r for r in recs}
check("marker VO exposes linkageId", all(by_id[m].get("linkageId") for m in (M[0], M[1])),
      str({m: by_id[m].get("linkageId") for m in M[:2]}))

section("D-2 cross-group merge")
st, pr = call("POST", "/api/marker_link/link", token=punctuate, body=[
    {"fromId": M[1], "toId": M[2]}])
check("second link 200", st == 200 and isinstance(pr, dict) and not pr.get("error"), str(pr)[:150])
grp = pr.get("data")
check("second link merges into new group", grp != grp1, f"grp1={grp1} grp2={grp}")
st, pr = call("POST", "/api/marker_link/get/list", token=punctuate, body={"groupIds": [grp]})
links = pr.get("data") or {}
flat = [l for lst in (links.values() if isinstance(links, dict) else [links]) for l in lst] if links else []
mine = [l for l in flat if l.get("fromId") in M and l.get("toId") in M]
check("both edges in merged group", len(mine) == 2, str(mine)[:200])

section("D-7 graph == list edges")
st, pr = call("POST", "/api/marker_link/get/graph", token=punctuate, body={"groupIds": [grp]})
graph = pr.get("data")
check("graph 200 with data", st == 200 and graph is not None, str(pr)[:150])
if isinstance(graph, dict):
    gv = (graph.get(grp) or {})
    rels = gv.get("relations") or {}
    check("graph relations keyed by markerId", set(rels.keys()) >= {str(M[0]), str(M[1]), str(M[2])}, str(list(rels.keys()))[:150])
    rel_refs = gv.get("relRefs") or {}
    check("graph relRefs populated", len(rel_refs) == len(mine), f"{len(rel_refs)} vs {len(mine)}")
    trig = next(iter(rel_refs.values()), {})
    check("graph relation TRIGGER shape", trig.get("type") == "TRIGGER" and trig.get("triggers"), str(trig)[:200])
else:
    check("graph is map", False, str(graph)[:150])

section("D-4 delete by group")
st, pr = call("DELETE", "/api/marker_link/delete", token=punctuate, body={"groupIds": [grp]})
check("delete by group 200", st == 200 and isinstance(pr, dict) and not pr.get("error"), str(pr)[:150])
st, pr = call("POST", "/api/marker_link/get/list", token=punctuate, body={"groupIds": [grp]})
links = pr.get("data") or {}
flat = [l for lst in (links.values() if isinstance(links, dict) else [links]) for l in lst] if links else []
mine = [l for l in flat if l.get("fromId") in M and l.get("toId") in M]
check("group gone after delete", not mine, str(mine)[:120])
st, pr = call("POST", "/api/marker/get/list_byid", token=admin, body=M)
recs = pr.get("data") or []
check("markers survive link delete", len(recs) == 4)

section("D-6 self-link rejected")
st, pr = call("POST", "/api/marker_link/link", token=punctuate, body=[
    {"fromId": M[0], "toId": M[0]}])
check("self link rejected", isinstance(pr, dict) and pr.get("error") is True, f"st={st} {str(pr)[:150]}")

section("D permission gates")
st, _ = call("POST", "/api/marker_link/link", token=user_t, body=[{"fromId": M[0], "toId": M[2]}])
check("MAP_USER link forbidden", st == 403, f"st={st}")
st, _ = call("POST", "/api/marker_link/get/list", token=user_t, body={"groupIds": [grp]})
check("MAP_USER link read ok", st == 200, f"st={st}")

section("cleanup")
st, pr = call("DELETE", f"/api/area/{AREA}", token=admin)
check("area cascade delete", st == 200 and isinstance(pr, dict) and not pr.get("error"))

sys.exit(summary())
