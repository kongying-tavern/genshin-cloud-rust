# 覆盖面：*_doc 的 md5 清单与 bin 载荷——gzip 解码、伪造 md5 拒绝、
# 角色过滤分页（MAP_USER < beta < admin）与越权取页拒绝。
"""Route E: *_doc md5/bin regression (encoding, roles, md5 gating).

注意：本套件的角色分页对比断言依赖库内已有 dev 量级数据（beta 专属页等），
全新空库上会以数据不足的形式失败，属预期（见 README 数据形状说明）。
"""
import pathlib
import sys
import gzip
import json

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from common import auth_ready

if not auth_ready():
    print("skipped: set REGRESSION_PASSWORD")
    sys.exit(0)

from common import login, call, raw_call, check, check_eq, section, summary

admin, _ = login("regress_admin")
user_t, _ = login("regress_user")
beta, _ = login("regress_beta")

def get_bin(path, token):
    status, headers, body = raw_call("GET", path, token=token)
    return status, headers, body

section("E-1/E-2 md5 lists + structure")
st, pr = call("GET", "/api/item_doc/list_page_bin_md5", token=admin, body=None)
entries = pr.get("data") if isinstance(pr, dict) else None
check("item_doc md5 list 200", st == 200 and isinstance(entries, list) and entries, str(pr)[:150])
if entries:
    e0 = entries[0]
    check("BinaryMd5Vo has md5+time", "md5" in e0 and "time" in e0, str(e0)[:120])

st, pr = call("GET", "/api/marker_doc/list_page_bin_md5", token=admin, body=None)
mentries = pr.get("data") if isinstance(pr, dict) else None
check("marker_doc md5 list 200", st == 200 and isinstance(mentries, list) and mentries, str(pr)[:150])

st, pr = call("GET", "/api/icon_doc/all_bin_md5", token=admin, body=None)
check("icon_doc single-object md5", st == 200 and isinstance(pr.get("data"), dict), str(pr)[:150])

st, pr = call("GET", "/api/marker_link_doc/all_list_bin_md5", token=admin, body=None)
check("marker_link_doc list md5 single-object", st == 200 and isinstance(pr.get("data"), dict), str(pr)[:150])

section("E-4 bin fetch + gzip decode")
if entries:
    md5 = entries[0]["md5"]
    st, headers, body = get_bin(f"/api/item_doc/list_page_bin/{md5}", admin)
    check("item bin 200 with gzip magic", st == 200 and isinstance(body, (bytes, bytearray)) and body[:2] == b"\x1f\x8b", f"st={st} head={bytes(body[:3]) if isinstance(body,(bytes,bytearray)) else body!r}")
    if isinstance(body, (bytes, bytearray)) and body[:2] == b"\x1f\x8b":
        payload = json.loads(gzip.decompress(body).decode("utf-8"))
        check("gzip payload is JSON list", isinstance(payload, list), type(payload).__name__)

# forged md5
st, headers, body = get_bin("/api/item_doc/list_page_bin/deadbeefdeadbeef", admin)
check("forged md5 -> business error", st == 200 and isinstance(body, dict) and body.get("error") is True, f"st={st} {str(body)[:120]}")

section("E-4 role-filtered md5 lists")
st, pr = call("GET", "/api/marker_doc/list_page_bin_md5", token=user_t, body=None)
u_entries = pr.get("data") or []
st, pr = call("GET", "/api/marker_doc/list_page_bin_md5", token=admin, body=None)
a_entries = pr.get("data") or []
check("MAP_USER md5 pages <= admin pages", len(u_entries) < len(a_entries), f"user={len(u_entries)} admin={len(a_entries)}")

st, pr = call("GET", "/api/item_doc/list_page_bin_md5", token=user_t, body=None)
ui = pr.get("data") or []
st, pr = call("GET", "/api/item_doc/list_page_bin_md5", token=beta, body=None)
bi = pr.get("data") or []
check("MAP_USER item md5 pages < beta pages", len(ui) < len(bi), f"user={len(ui)} beta={len(bi)}")

# MAP_USER cannot fetch a beta-only page even with a known md5
beta_only = [e["md5"] for e in bi if e not in ui]
if beta_only:
    st, headers, body = get_bin(f"/api/item_doc/list_page_bin/{beta_only[0]}", user_t)
    check("MAP_USER cannot fetch beta page by md5", st == 200 and isinstance(body, dict) and body.get("error") is True, f"st={st} {str(body)[:120]}")
else:
    check("beta-only pages exist for cross-role test", False, "no beta-only md5 page found")

section("E marker_doc page consistency (id/3000)")
if mentries:
    # admin sees groups; normal group may be split into multiple pages
    # 依赖 dev 库的点位数据规模（TODO: 数据规模断言依赖库内数据形状）。
    check("marker_doc has multiple pages (large dataset)", len(mentries) >= 2, f"pages={len(mentries)}")

sys.exit(summary())
