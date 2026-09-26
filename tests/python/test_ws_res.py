# 覆盖面：WebSocket 握手鉴权（无 token 拒绝、userId 与 token 主体不一致拒绝）、
# 业务事件广播（MarkerAdded / NoticeAdded / BinaryPurged 防抖、W 载荷形态）、
# res 图片上传（最小 PNG 上传成功、非图片拒绝，依赖本地 MinIO）。
"""WebSocket broadcast events + res upload regression."""
import pathlib
import sys
import asyncio
import json
import time
import urllib.error
import urllib.request

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from common import auth_ready

if not auth_ready():
    print("skipped: set REGRESSION_PASSWORD")
    sys.exit(0)

try:
    import websockets  # noqa: F401
except ImportError:
    print("skipped: python 包 websockets 未安装（pip install websockets>=14）")
    sys.exit(0)

from common import BASE, UA, login, call, check, section, summary

admin, _ = login("regress_admin")
manager, _ = login("regress_manager")
punctuate, resp_punc = login("regress_punctuate")
user_tok, resp_user = login("regress_user")
PUNC_ID = resp_punc["userId"]
# WS 鉴权：路径 userId 必须与 token 主体一致，token 经 ?token= 查询参数传递
# （浏览器 WebSocket API 无法自定义请求头时的通道）。
WS_UID = resp_user["userId"]

WS = BASE.replace("http", "ws")


def _handshake_status(e):
    """从 websockets 的握手失败异常里取 HTTP 状态码（v14+ 为
    InvalidStatus.response.status_code，旧版为 InvalidStatus.status_code）。"""
    r = getattr(e, "response", None)
    return getattr(r, "status_code", None) or getattr(e, "status_code", None)


async def ws_negotiate(uri):
    """返回 "connected" 或 ("rejected", status)。"""
    try:
        async with websockets.connect(uri, open_timeout=10, additional_headers={"User-Agent": UA}):
            return "connected"
    except Exception as e:
        return "rejected", _handshake_status(e)


async def ws_run():
    events = []

    # ── 握手鉴权负向探测（无 token → 401；userId 冒用他人 → 403）──
    r = await ws_negotiate(f"{WS}/ws/{WS_UID}")
    check("WS handshake without token rejected (401)",
          r[0] == "rejected" and r[1] == 401, repr(r))
    r = await ws_negotiate(f"{WS}/ws/{PUNC_ID}?token={user_tok}")
    check("WS handshake with mismatched userId rejected (403)",
          r[0] == "rejected" and r[1] == 403, repr(r))

    # ── 带本人 token 的合法连接 ──
    uri = f"{WS}/ws/{WS_UID}?token={user_tok}"
    async with websockets.connect(uri, additional_headers={"User-Agent": UA}) as ws:
        await asyncio.sleep(0.5)

        # setup: own area/item/marker -> MarkerAdded event
        st, pr = call("PUT", "/api/area/add", token=manager, body={
            "name": "regress_ws_area", "parentId": -1, "isFinal": True, "iconId": 0, "hiddenFlag": 0})
        area = pr["data"]
        st, pr = call("PUT", "/api/item/add", token=manager, body={
            "areaId": area, "name": "regress_ws_item", "typeIdList": [], "iconId": 0,
            "iconStyleType": 0, "hiddenFlag": 0})
        item = pr["data"]
        st, pr = call("PUT", "/api/marker/single", token=punctuate, body={
            "content": "ws", "extra": None, "hiddenFlag": 0,
            "itemList": [{"itemId": item, "count": 1}], "markerCreatorId": PUNC_ID,
            "markerTitle": "regress_ws_m", "picture": None, "pictureCreatorId": None,
            "position": "1.0,1.0", "refreshTime": 0, "videoPath": None})
        mid = pr["data"]["id"] if isinstance(pr.get("data"), dict) else pr["data"]

        # notice write -> NoticeAdded (admin)
        st, pr = call("PUT", "/api/notice/add", token=admin, body={
            "content": "regress ws notice", "title": "regress_notice_ws", "channel": ["COMMON"]})

        # collect events for a window covering the debounce purge
        deadline = time.time() + 45
        while time.time() < deadline:
            try:
                msg = await asyncio.wait_for(ws.recv(), timeout=max(0.1, deadline - time.time()))
                events.append(json.loads(msg))
            except asyncio.TimeoutError:
                break

        # cleanup
        call("DELETE", f"/api/area/{area}", token=admin)
        try:
            st, pr = call("POST", "/api/notice/get/list", token=admin, body={})
            notices = pr.get("data") or []
            if isinstance(notices, dict):
                notices = notices.get("record") or []
            for n in notices:
                if n.get("title") == "regress_notice_ws":
                    call("DELETE", f"/api/notice/{n['id']}", token=admin)
        except Exception as e:
            print("  cleanup notice err:", e)
        return events, mid


section("WS broadcast events")
events, mid = asyncio.run(ws_run())
names = [e.get("event") for e in events]
print("  events received:", names)
check("MarkerAdded broadcast", "MarkerAdded" in names, str(names))
check("NoticeAdded broadcast", "NoticeAdded" in names, str(names))
check("BinaryPurged (debounced) broadcast", any("Purged" in (n or "") for n in names), str(names))
w = next((e for e in events if e.get("event") == "MarkerAdded"), None)
if w:
    check("W payload shape {event,message,data,time}",
          all(k in w for k in ("event", "message", "data", "time")), str(w)[:150])

section("res upload (local MinIO)")
import uuid as _uuid


def file_form(path, filename, content, ctype):
    b = _uuid.uuid4().hex
    head = (
        f"--{b}\r\n"
        f'Content-Disposition: form-data; name="filePath"\r\n\r\n{path}\r\n'
        f"--{b}\r\n"
        f'Content-Disposition: form-data; name="file"; filename="{filename}"\r\n'
        f"Content-Type: {ctype}\r\n\r\n"
    ).encode()
    return head + content + f"\r\n--{b}--\r\n".encode(), f"multipart/form-data; boundary={b}"


# minimal PNG: magic + IHDR + IEND with valid CRCs
import struct as _st
from zlib import crc32 as _crc
_ihdr = _st.pack(">II5B", 1, 1, 8, 2, 0, 0, 0)
png = (
    b"\x89PNG\r\n\x1a\n"
    + _st.pack(">I", 13) + b"IHDR" + _ihdr + _st.pack(">I", _crc(b"IHDR" + _ihdr) & 0xFFFFFFFF)
    + _st.pack(">I", 0) + b"IEND" + _st.pack(">I", _crc(b"IEND") & 0xFFFFFFFF)
)
body, ct = file_form("regression/ws_test.png", "ws_test.png", png, "image/png")
req = urllib.request.Request(BASE + "/api/res/upload/image", data=body, method="PUT",
                             headers={"Content-Type": ct, "User-Agent": UA,
                                      "Authorization": f"Bearer {punctuate}"})
try:
    with urllib.request.urlopen(req, timeout=30) as r:
        up = json.loads(r.read().decode())
    check("res upload 200", r.status == 200 and not up.get("error"), str(up)[:150])
    data = up.get("data") or {}
    if isinstance(data, list) and data:
        check("upload returns fileUrl", bool(data[0].get("fileUrl") or data[0].get("url")), str(data)[:200])
    elif isinstance(data, dict):
        check("upload returns fileUrl", bool(data.get("fileUrl") or data.get("url")), str(data)[:200])
except urllib.error.HTTPError as e:
    check("res upload 200", False, f"{e.code} {e.read().decode()[:200]}")
except Exception as e:
    check("res upload 200", False, str(e)[:200])

# non-image rejected
body, ct = file_form("regression/ws_test.txt", "ws_test.txt", b"not an image", "text/plain")
req = urllib.request.Request(BASE + "/api/res/upload/image", data=body, method="PUT",
                             headers={"Content-Type": ct, "User-Agent": UA,
                                      "Authorization": f"Bearer {punctuate}"})
try:
    with urllib.request.urlopen(req, timeout=30) as r:
        up = json.loads(r.read().decode())
    check("non-image rejected in R", up.get("error") is True, str(up)[:150])
except urllib.error.HTTPError as e:
    check("non-image rejected in R", 400 <= e.code < 500, f"{e.code}")

sys.exit(summary())
