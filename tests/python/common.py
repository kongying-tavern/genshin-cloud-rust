"""Shared harness for the live API regression suite (tests/python).

运行前置（详见同目录 README.md）：
- 本地起 Rust 后端（默认 ``http://127.0.0.1:8101``，可用 ``REGRESSION_BASE``
  覆盖）；
- 数据库可用 ``tests/docker/docker-compose.e2e.yml`` 起一套本地库，或指向
  dev 库（部分断言依赖 dev 量级的数据，详见 README）；
- 预先种子 ``regress_admin`` / ``regress_manager`` / ``regress_beta`` /
  ``regress_punctuate`` / ``regress_user`` / ``regress_newuser`` 六个账号
  （role 分别为 admin/manager/beta/punctate/user/user），口令一致且等于
  ``REGRESSION_PASSWORD``；
- ``REGRESSION_PASSWORD`` 必须设置。敏感信息红线：该口令曾用于共享开发库，
  一律经环境变量注入，任何真实口令禁止硬编码入库。

套件创建的一切数据均以 ``regress`` 前缀命名，并在各套件结束时清理。
"""

import json
import os
import urllib.error
import urllib.request
import uuid

BASE = os.environ.get("REGRESSION_BASE", "http://127.0.0.1:8101")
# 无默认值：未设置时各套件经 auth_ready() 自跳过（敏感信息红线，口令不入库）。
PASSWORD = os.environ.get("REGRESSION_PASSWORD", "")
UA = "regression/1.0"

PASS, FAIL, SKIP = [], [], []


def auth_ready() -> bool:
    """PASSWORD 已设置且非空时返回 True；各套件开头据此自跳过。"""
    return bool(PASSWORD.strip())


def multipart(fields):
    boundary = uuid.uuid4().hex
    chunks = []
    for k, v in fields.items():
        chunks.append(f"--{boundary}")
        chunks.append(f'Content-Disposition: form-data; name="{k}"')
        chunks.append("")
        chunks.append(str(v))
    chunks.append(f"--{boundary}--")
    return b"\r\n".join(c.encode("utf-8") for c in chunks), f"multipart/form-data; boundary={boundary}"


def raw_call(method, path, token=None, body=None, form=None, headers=None, timeout=240):
    """Returns (status, headers, parsed_json_or_text)."""
    url = BASE + path
    hdrs = {"User-Agent": UA}
    if token:
        hdrs["Authorization"] = f"Bearer {token}"
    if headers:
        hdrs.update(headers)
    data = None
    if form is not None:
        data, ct = multipart(form)
        hdrs["Content-Type"] = ct
    elif body is not None:
        data = json.dumps(body).encode("utf-8")
        hdrs["Content-Type"] = "application/json"
    req = urllib.request.Request(url, data=data, headers=hdrs, method=method)
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            raw = resp.read()
            status, rh = resp.status, dict(resp.headers)
    except urllib.error.HTTPError as e:
        raw = e.read()
        status, rh = e.code, dict(e.headers)
    try:
        parsed = json.loads(raw.decode("utf-8")) if raw else None
    except (json.JSONDecodeError, UnicodeDecodeError):
        parsed = raw
    return status, rh, parsed


def call(method, path, token=None, body=None, form=None, headers=None, timeout=240):
    status, _, parsed = raw_call(method, path, token, body, form, headers, timeout)
    return status, parsed


def login(username, password=None):
    """登录并返回 ``(access_token, 完整响应 dict)``；响应含 ``userId`` /
    ``userRoles`` / ``expires_in`` 等字段（Java SysToken 契约）。
    """
    if password is None:
        password = PASSWORD
    body, ct = multipart({"grant_type": "password", "username": username, "password": password})
    req = urllib.request.Request(
        BASE + "/oauth/token", data=body, headers={"Content-Type": ct, "User-Agent": UA}
    )
    try:
        with urllib.request.urlopen(req, timeout=20) as resp:
            data = json.loads(resp.read().decode("utf-8"))
    except urllib.error.HTTPError as e:
        raise RuntimeError(
            f"login({username!r}) failed: HTTP {e.code} {e.read().decode('utf-8', 'replace')[:200]}"
            f" —— 检查 REGRESSION_BASE（当前 {BASE}）与账号种子"
            f"（regress_* 账号须已存在，口令须与 REGRESSION_PASSWORD 一致）"
        ) from e
    except urllib.error.URLError as e:
        raise RuntimeError(
            f"login({username!r}) cannot reach {BASE}: {e.reason}"
            f" —— 检查 REGRESSION_BASE 与本地后端是否已启动"
        ) from e
    return data["access_token"], data


def check(name, cond, detail=""):
    if cond:
        PASS.append(name)
        print(f"  ok   {name}")
    else:
        FAIL.append(name)
        print(f"  FAIL {name}  {detail}")


def check_eq(name, actual, expected):
    check(name, actual == expected, f"expected={expected!r} actual={actual!r}")


def section(title):
    print(f"\n== {title} ==")


def summary():
    print(f"\n==== SUMMARY: {len(PASS)} passed, {len(FAIL)} failed, {len(SKIP)} skipped ====")
    for f in FAIL:
        print(f"  FAILED: {f}")
    return len(FAIL)
