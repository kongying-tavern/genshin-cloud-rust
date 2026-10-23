# 覆盖面：种子 regress_* 六账号（管理员经邀请码通道创建），供整个回归套件使用。
"""Seed the six regress_* accounts over HTTP (no direct DB access).

Uses the admin account (INIT_ADMIN_* / seed defaults) to create one
invitation per target role, then consumes each invitation with the shared
REGRESSION_PASSWORD. Idempotent: existing invitations/users are skipped.

Usage:
    REGRESSION_PASSWORD=... python tests/python/seed_accounts.py
"""
import json
import os
import pathlib
import sys
import urllib.error
import urllib.request

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

from common import BASE, UA, PASSWORD, call, login, multipart

# (username, role_id) —— role 对照见 README 的种子账号表。
TARGETS = [
    ("regress_admin", 0),
    ("regress_manager", 1),
    ("regress_beta", 2),
    ("regress_punctuate", 3),
    ("regress_user", 4),
    ("regress_newuser", 4),
]


def main() -> int:
    if not PASSWORD:
        print("skipped: set REGRESSION_PASSWORD")
        return 0
    # 管理员通道：CI 用 init_db 的 INIT_ADMIN_* 播种 regress_admin（邀请码
    # 通道明确拒绝授予 admin 角色）；本地 dev 库通常是 admin/admin123。
    admin_candidates = [
        (os.environ.get("INIT_ADMIN_USERNAME", "regress_admin"),
         os.environ.get("INIT_ADMIN_PASSWORD", PASSWORD)),
        ("admin", "admin123"),
    ]
    admin = None
    for uname, upass in admin_candidates:
        try:
            body, ct = multipart({"grant_type": "password", "username": uname, "password": upass})
            req = urllib.request.Request(
                BASE + "/oauth/token", data=body, headers={"Content-Type": ct, "User-Agent": UA}
            )
            with urllib.request.urlopen(req, timeout=20) as r:
                admin = json.loads(r.read().decode())["access_token"]
            print(f"  admin channel: {uname}")
            break
        except urllib.error.HTTPError:
            continue
        except urllib.error.URLError as e:
            print(f"cannot reach {BASE}: {e.reason}")
            return 1
    if admin is None:
        print("admin login failed for every candidate; seed an admin with init_db (INIT_ADMIN_*)")
        return 1
    ok = True
    for username, role in TARGETS:
        # 幂等探测走管理员用户检索——**不要**用试登录：登录失败按 IP 限流
        # （5 次/分钟），连续探测 6 个账号会把后续正常登录一起打成 400。
        st, pr = call("POST", "/system/user/info/userList", token=admin, body={
            "username": username, "current": 1, "size": 1,
        })
        record = ((pr or {}).get("data") or {}).get("record") if isinstance(pr, dict) else None
        if st == 200 and record:
            print(f"  {username}: already seeded — skip")
            continue
        st, pr = call("POST", "/system/invitation/update", token=admin, body={
            "code": f"seed_{username}", "username": username,
            "roleId": role, "remark": "regression seed", "accessPolicy": [],
        })
        if not (st == 200 and isinstance(pr, dict) and not pr.get("error")):
            msg = str(pr.get("message") if isinstance(pr, dict) else pr)[:120]
            if role == 0:
                print(f"  {username}: admin role cannot be granted by invitation ({msg}) — seed it via init_db's INIT_ADMIN_USERNAME/INIT_ADMIN_PASSWORD")
            else:
                print(f"  {username}: invitation create failed st={st} {msg}")
            ok = False
            continue
        st, pr = call("POST", "/system/invitation/consume", token=admin, body={
            "code": f"seed_{username}", "username": username,
            "password": PASSWORD, "nickname": f"{username} nick",
        })
        if st == 200 and isinstance(pr, dict) and not pr.get("error"):
            print(f"  {username}: seeded (role {role})")
        else:
            print(f"  {username}: consume failed st={st} {str(pr)[:150]}")
            ok = False
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
