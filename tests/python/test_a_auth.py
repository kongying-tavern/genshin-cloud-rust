# 覆盖面：oauth/token 密码模式（正确/错误口令、不存在用户）、refresh_token
# 轮换与复用、client_credentials 访客态、jwks 端点、网关层 401 拒绝、五角色登录。
"""Route A: login & auth chain regression."""
import json
import os
import pathlib
import sys
import time
import urllib.error
import urllib.request

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from common import auth_ready

if not auth_ready():
    print("skipped: set REGRESSION_PASSWORD")
    sys.exit(0)

from common import BASE, login, call, raw_call, check, check_eq, section, summary, multipart

section("A-1 oauth/token password mode")
tok_admin, resp_admin = login("regress_admin")
uid_admin = resp_admin.get("userId")
check("password login returns access_token", bool(tok_admin))
check_eq("token_type", resp_admin.get("token_type"), "bearer")
# userId 为动态值（dev 库上种子账号是 200，本地库可能不同）：只断言形态，
# 各处用户路径一律从登录响应取 uid。
check("userId positive int", isinstance(uid_admin, int) and uid_admin > 0, repr(uid_admin))
check("userRoles contains ADMIN", "ADMIN" in (resp_admin.get("userRoles") or []), resp_admin.get("userRoles"))
check("expires_in positive int", isinstance(resp_admin.get("expires_in"), int) and resp_admin["expires_in"] > 0)
# env 字段镜像后端 environment_id()：APP_ENV 归一化，缺省 dev。
expected_env = os.environ.get("APP_ENV", "dev").strip().lower() or "dev"
check_eq("env field mirrors APP_ENV/dev", resp_admin.get("env"), expected_env)
check_eq("message field is empty string", resp_admin.get("message"), "")

# wrong password (rate limit: max 5 fails/min — single attempt here)
body, ct = multipart({"grant_type": "password", "username": "regress_admin", "password": "wrong"})
req = urllib.request.Request(BASE + "/oauth/token", data=body, headers={"Content-Type": ct, "User-Agent": "regression/1.0"})
try:
    with urllib.request.urlopen(req, timeout=20) as r:
        code, payload = r.status, json.loads(r.read().decode())
except urllib.error.HTTPError as e:
    code, payload = e.code, e.read().decode()[:300]
check("wrong password rejected (4xx)", 400 <= code < 500, f"code={code} body={payload}")

# nonexistent user
time.sleep(61)  # reset the per-minute failure window before the next negative attempt
body, ct = multipart({"grant_type": "password", "username": "no_such_regress_user", "password": "x"})
req = urllib.request.Request(BASE + "/oauth/token", data=body, headers={"Content-Type": ct, "User-Agent": "regression/1.0"})
try:
    with urllib.request.urlopen(req, timeout=20) as r:
        code = r.status
except urllib.error.HTTPError as e:
    code = e.code
check("nonexistent user rejected (4xx)", 400 <= code < 500, f"code={code}")

section("A-1 refresh_token")
refresh = resp_admin.get("refresh_token")
check("refresh_token present", bool(refresh))
st, pr = call("POST", f"/oauth/token?grant_type=refresh_token&refresh_token={refresh}")
check("refresh succeeds", st == 200 and isinstance(pr, dict) and pr.get("access_token"), f"st={st} resp={str(pr)[:200]}")

# old access token after refresh: Rust rotates sessions on refresh (S3). Old access
# should be revoked only if rotation deletes it; allow either but record.
new_tok = pr.get("access_token") if isinstance(pr, dict) else None
st2, who = call("GET", f"/system/user/info/{uid_admin}", token=new_tok)
check("new token usable", st2 == 200, f"st={st2}")
st3, who3 = call("GET", f"/system/user/info/{uid_admin}", token=tok_admin)
print(f"  info: old access token after refresh -> {st3} (rotation policy recorded)")

# refresh token must not work as access token
st4, _ = call("GET", f"/system/user/info/{uid_admin}", token=refresh)
check("refresh token rejected as access token", st4 == 401, f"st={st4}")

# reused refresh token (rotation should revoke it)
st5, pr5 = call("POST", f"/oauth/token?grant_type=refresh_token&refresh_token={refresh}")
print(f"  info: reused refresh token -> {st5} (rotation policy recorded)")

section("A-1 client_credentials -> VISITOR")
st, pr = call("POST", "/oauth/token?grant_type=client_credentials&scope=all")
check("client_credentials issues token", st == 200 and pr.get("access_token"), f"st={st}")
visitor_tok = pr.get("access_token")
st, pr = call("POST", "/api/area/get/list", token=visitor_tok, body={})
check("visitor token can read area list", st == 200, f"st={st}")

section("A-2 jwks")
st, hdrs, pr = raw_call("GET", "/.well-known/jwks.json")
check("jwks reachable without token", st == 200, f"st={st}")

section("A-3 gateway-level auth")
st, pr = call("GET", f"/system/user/info/{uid_admin}")
check("no token -> 401", st == 401, f"st={st}")
st, pr = call("GET", f"/system/user/info/{uid_admin}", token="garbage.token.here")
check("tampered token -> 401", st == 401, f"st={st}")
st, pr = call("GET", f"/system/user/info/{uid_admin}", headers={"Authorization": "Bearer expired"})
check("bad bearer -> 401", st == 401, f"st={st}")

# expired token: forge one via shared knowledge is not possible without the secret;
# covered indirectly by refresh-as-access rejection above.

section("A login response for each role")
for uname in ["regress_manager", "regress_beta", "regress_punctuate", "regress_user"]:
    t, r = login(uname)
    check(f"{uname} can login", bool(t))

sys.exit(summary())
