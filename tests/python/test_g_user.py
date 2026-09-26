# 覆盖面：用户域——用户信息读取的字段可见性、自助改名、越权与提权拦截、
# 改口令、注册（MANAGER+）、角色表、设备列表、个人档案八件套、邀请码全流程、
# 公告读、app 触发器、res 探测与缓存清理端点。
"""Routes G/F/H: user domain, invitation, archive, role, notice, cache, res."""
import pathlib
import sys
import json

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from common import auth_ready

if not auth_ready():
    print("skipped: set REGRESSION_PASSWORD")
    sys.exit(0)

from common import PASSWORD, login, call, check, check_eq, section, summary

admin, resp_admin = login("regress_admin")
manager, _ = login("regress_manager")
user_t, resp_user = login("regress_user")
user2, resp_user2 = login("regress_beta")  # another non-admin account
punctuate, _ = login("regress_punctuate")
# 用户 id 一律从登录响应动态取（dev 库上 admin=200/beta=202/user=204）。
UID_ADMIN = resp_admin["userId"]
UID_BETA = resp_user2["userId"]
UID_USER = resp_user["userId"]

section("G-1 user info / update / password")
st, pr = call("GET", f"/system/user/info/{UID_ADMIN}", token=admin)
check("admin reads any user", st == 200 and isinstance(pr.get("data"), dict), str(pr)[:150])
check("user VO roleId numeric", isinstance((pr.get("data") or {}).get("roleId"), int), str(pr.get("data"))[:150])

st, pr = call("GET", f"/system/user/info/{UID_BETA}", token=user_t)
d = pr.get("data") or {}
check("USER reading others -> public fields only", st == 200 and d.get("qq") is None and d.get("phone") is None and not d.get("accessPolicy"), str(d)[:150])
st, pr = call("GET", f"/system/user/info/{UID_USER}", token=user_t)
check("USER reads self", st == 200 and isinstance(pr.get("data"), dict), f"st={st}")

st, pr = call("POST", "/system/user/update", token=user_t, body={"userId": UID_USER, "nickname": "regress_renamed"})
check("user updates self", st == 200 and isinstance(pr, dict) and not pr.get("error"), str(pr)[:150])
st, pr = call("GET", f"/system/user/info/{UID_USER}", token=user_t)
check("rename persisted", (pr.get("data") or {}).get("nickname") == "regress_renamed", str(pr.get("data"))[:120])

st, pr = call("POST", "/system/user/update", token=user_t, body={"userId": UID_BETA, "nickname": "hijack"})
check("user updating others rejected", st in (401, 403) or (isinstance(pr, dict) and pr.get("error")), f"st={st} {str(pr)[:100]}")

# role escalation attempt: MAP_USER promotes self to admin
st, pr = call("POST", "/system/user/update", token=user_t, body={"userId": UID_USER, "roleId": 0})
st2, pr2 = call("GET", f"/system/user/info/{UID_USER}", token=admin)
# roleId 4 == MAP_USER（_utils::types::SystemUserRole 枚举数值，schema 常量）
check("role escalation blocked", (pr2.get("data") or {}).get("roleId") == 4, str(pr2.get("data"))[:120])

st, pr = call("POST", "/system/user/update_password", token=user_t,
              body={"userId": UID_USER, "oldPassword": "wrong_pw", "newPassword": "NewPass#1"})
check("wrong old password rejected", isinstance(pr, dict) and pr.get("error") is True, str(pr)[:150])

st, pr = call("POST", "/system/user/update_password", token=user_t,
              body={"userId": UID_BETA, "oldPassword": PASSWORD, "newPassword": "x"})
check("password change for other user rejected", st in (401, 403) or (isinstance(pr, dict) and pr.get("error")), f"st={st}")

section("G-1 register (MANAGER+)")
# 注册口令沿用 REGRESSION_PASSWORD（已满足口令策略；敏感信息红线——不发明硬编码口令）。
st, pr = call("POST", "/system/user/register", token=manager, body={
    "username": "regress_newuser", "password": PASSWORD, "nickname": "新注册"})
check("register 200 (or already exists from prior run)",
      st == 200 and (not pr.get("error") or "exists" in str(pr.get("message"))), str(pr)[:150])
st, pr = call("POST", "/system/user/register", token=manager, body={
    "username": "regress_newuser", "password": PASSWORD})
check("duplicate username rejected", isinstance(pr, dict) and pr.get("error") is True, str(pr)[:120])
st, pr = call("POST", "/system/user/register", token=user_t, body={"username": "x", "password": "y"})
check("MAP_USER register forbidden", st == 403, f"st={st}")

section("G-3 role/list + device")
st, pr = call("GET", "/system/role/list", token=user_t)
roles = pr.get("data") or []
# 6 个角色 = SystemUserRole 枚举全集（0..5），schema 常量。
check("role list has 6 roles", isinstance(roles, list) and len(roles) == 6, str(roles)[:200])
st, pr = call("POST", "/system/device/list", token=admin, body={"userId": UID_ADMIN})
check("device list 200", st == 200, f"st={st} {str(pr)[:100]}")

section("G-3 archive 八件套")
st, pr = call("GET", "/system/archive/last/0", token=user_t)
check("archive last empty slot", st == 200, f"st={st} {str(pr)[:100]}")
payload = json.dumps({"big": "x" * 5000, "unicode": "中文🎉"}, ensure_ascii=False)
st, pr = call("POST", "/system/archive/save/0", token=user_t, body=json.loads(payload) if False else {"big": "x" * 5000, "unicode": "中文🎉", "name": "regress_slot"})
check("archive save 200", st == 200 and isinstance(pr, dict) and not pr.get("error"), str(pr)[:150])
st, pr = call("GET", "/system/archive/last/0", token=user_t)
got = (pr.get("data") or {}) if isinstance(pr.get("data"), dict) else {}
check("archive data roundtrip", (got.get("archive") or "").find("中文") >= 0, str(got)[:150])
# duplicate save -> idempotent false
st, pr = call("POST", "/system/archive/save/0", token=user_t, body=json.loads(payload) if False else {"big": "x" * 5000, "unicode": "中文🎉", "name": "regress_slot"})
check("duplicate save idempotent", isinstance(pr, dict) and pr.get("data") is False, str(pr)[:120])
# cross-user isolation
st, pr = call("GET", "/system/archive/last/0", token=user2)
got2 = (pr.get("data") or {}) if isinstance(pr.get("data"), dict) else {}
check("archive isolated across users", not got2.get("data"), str(got2)[:100])
st, pr = call("DELETE", "/system/archive/slot/0", token=user_t)
check("archive delete slot", st == 200, f"st={st}")

import time as _t
_run = str(int(_t.time()) % 100000)
section("G-2 invitation")
# 邀请码口令同样沿用 REGRESSION_PASSWORD（无硬编码口令入库）。
st, pr = call("POST", "/system/invitation/update", token=admin, body={
    "code": "RGXINV" + _run, "username": "regress_invitee" + _run, "roleId": 4, "accessPolicy": [], "remark": ""})
check("invitation update 200", st == 200 and isinstance(pr, dict) and not pr.get("error"), str(pr)[:150])
st, pr = call("POST", "/system/invitation/info", body={"code": "RGXINV" + _run, "username": "regress_invitee" + _run})
check("invitation info (no auth)", st == 200 and isinstance(pr.get("data"), dict), str(pr)[:150])
st, pr = call("POST", "/system/invitation/consume", body={"code": "RGXINV" + _run, "username": "regress_invitee" + _run, "password": PASSWORD})
check("invitation consume SUCCESS", st == 200 and isinstance(pr, dict) and not pr.get("error"), str(pr)[:150])
st, pr = call("POST", "/system/invitation/consume", body={"code": "RGXINV" + _run, "username": "regress_invitee2" + _run, "password": PASSWORD})
check("invitation second consume rejected", isinstance(pr, dict) and pr.get("error") is True, str(pr)[:120])
try:
    t, r = login("regress_invitee" + _run, PASSWORD)
    check("invitee can login", bool(t))
except Exception as e:
    check("invitee can login", False, str(e)[:120])
# TODO: 邀请产出账号（regress_invitee*/regress_newuser）暂无删除端点，
# 由运维在库里定期清理（username LIKE 'regress_%'）。

section("F-1 notice")
st, pr = call("POST", "/api/notice/get/list", token=user_t, body={})
check("notice list (public read)", st == 200 and isinstance(pr, dict), str(pr)[:100])

section("F-2 app trigger (ADMIN)")
st, pr = call("POST", "/api/app/trigger/update", token=admin)
check("app trigger 200 (admin)", st == 200, f"st={st} {str(pr)[:100]}")

section("H res")
st, pr = call("GET", "/api/res/get?path=nonexistent_regress.png")
print(f"  info: res/get nonexistent -> {st} {str(pr)[:80]}")

section("cache endpoints")
st, pr = call("DELETE", "/api/cache/marker", token=admin)
check("cache marker clear 200", st == 200, f"st={st}")
st, pr = call("DELETE", "/api/cache/iconTag", token=admin, body=[])
check("cache icon (empty=all) 200", st == 200, f"st={st}")

sys.exit(summary())
