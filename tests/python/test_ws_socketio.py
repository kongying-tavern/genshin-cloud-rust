# 覆盖面：Socket.IO 兼容通道（`/socket.io`，socketioxide）——握手鉴权矩阵、
# `message` 事件信封（MarkerAdded / MarkerBinaryPurged）、`rttcheck` 回环。
# 依赖 `python-socketio`（websocket 传输）；未安装时自跳过。
"""Socket.IO compatibility regression (handshake auth, message envelope, rttcheck)."""
import pathlib
import sys
import asyncio
import json
import time

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from common import auth_ready

if not auth_ready():
    print("skipped: set REGRESSION_PASSWORD")
    sys.exit(0)

try:
    import socketio  # noqa: F401
except ImportError:
    print("skipped: python 包 python-socketio 未安装（pip install python-socketio）")
    sys.exit(0)

from common import BASE, UA, login, call, check, section, summary

admin, resp_admin = login("regress_admin")
punctuate, resp_punc = login("regress_punctuate")
PUNC_ID = resp_punc["userId"]
ADMIN_ID = resp_admin["userId"]

import socketio as _sio


async def ws_connect(token, uid):
    """以 socket.io-client 兼容方式连接（websocket 传输 + query 参数）。"""
    sio = _sio.AsyncClient(ssl_verify=False)
    events = []

    @sio.on("message")
    async def on_message(message):
        events.append(json.loads(message))

    await sio.connect(
        f"{BASE}?userId={uid}&token={token}",
        socketio_path="/socket.io",
        transports=["websocket"],
        headers={"User-Agent": UA},
    )
    return sio, events


async def rttcheck_roundtrip(sio):
    """rttcheck 回环：回包带 id / receiveTimestamp / sendTimestamp。"""
    got = {}

    @sio.on("rttcheck")
    async def on_rttcheck(data):
        got.update(data or {})

    # 首包可能早于服务端按连接注册处理器（connect ack 先行），重发至回包。
    deadline = time.time() + 5
    while time.time() < deadline and "sendTimestamp" not in got:
        await sio.emit("rttcheck", {"id": "regress_rtt"})
        loop_deadline = time.time() + 0.5
        while time.time() < loop_deadline and "sendTimestamp" not in got:
            await asyncio.sleep(0.1)
    return got


async def ws_reject_cases():
    """无 token → 服务端断开；userId 冒用 → 服务端断开。"""
    results = []
    for label, url in [
        ("no token", f"{BASE}?userId={ADMIN_ID}"),
        ("mismatched subject", f"{BASE}?userId=999999&token={punctuate}"),
    ]:
        sio = _sio.AsyncClient(ssl_verify=False)
        rejected = False

        @sio.on("disconnect")
        async def on_disc():
            nonlocal rejected
            rejected = True

        try:
            await sio.connect(url, socketio_path="/socket.io", transports=["websocket"],
                              headers={"User-Agent": UA})
            await asyncio.sleep(1.0)
            rejected = rejected or not sio.connected
        except Exception:
            rejected = True
        finally:
            try:
                await sio.disconnect()
            except Exception:
                pass
        results.append((label, rejected))
    return results


async def main_flow():
    sio, events = await ws_connect(punctuate, PUNC_ID)
    if not sio.connected:
        check("socket.io connect (valid token)", False, "not connected")
        return
    check("socket.io connect (valid token)", True)

    # rttcheck 回环
    got = await rttcheck_roundtrip(sio)
    check("rttcheck echo carries timestamps",
          "sendTimestamp" in got and "receiveTimestamp" in got, str(got)[:120])

    # setup: area + item + marker → MarkerAdded 信封（message 事件，JSON 字符串）
    st, pr = call("PUT", "/api/area/add", token=admin, body={
        "name": "regress_sio_area", "code": "SIO", "parentId": -1, "isFinal": True,
        "iconId": 0, "hiddenFlag": 0})
    area = pr["data"]
    st, pr = call("PUT", "/api/item/add", token=admin, body={
        "areaId": area, "name": "regress_sio_item", "typeIdList": [], "iconId": 0,
        "iconStyleType": 0, "hiddenFlag": 0})
    item = pr["data"]
    st, pr = call("PUT", "/api/marker/single", token=punctuate, body={
        "content": "sio", "extra": None, "hiddenFlag": 0,
        "itemList": [{"itemId": item, "count": 1}], "markerCreatorId": PUNC_ID,
        "markerTitle": "regress_sio_m", "picture": None, "pictureCreatorId": None,
        "position": "1.0,1.0", "refreshTime": 0, "videoPath": None})
    mid = pr["data"]["id"] if isinstance(pr.get("data"), dict) else pr["data"]

    # 缓存失效：确定性非节流通道（DELETE /api/cache/marker → MarkerBinaryPurged）
    call("DELETE", "/api/cache/marker", token=admin)

    deadline = time.time() + 10
    while time.time() < deadline:
        await asyncio.sleep(0.1)
        names = [e.get("event") for e in events]
        if "MarkerBinaryPurged" in names:
            break

    names = [e.get("event") for e in events]
    check("socket.io message envelope MarkerAdded", "MarkerAdded" in names, str(names)[:200])
    added = next((e for e in events if e.get("event") == "MarkerAdded"), None)
    if added:
        check("message envelope carries W shape",
              all(k in added for k in ("event", "message", "data", "time")), str(added)[:150])
    check("socket.io message MarkerBinaryPurged", "MarkerBinaryPurged" in names, str(names)[:200])

    call("DELETE", f"/api/area/{area}", token=admin)
    await sio.disconnect()


section("Socket.IO handshake auth matrix")
for label, rejected in asyncio.run(ws_reject_cases()):
    check(f"socket.io {label} rejected", rejected, label)

section("Socket.IO message flow")
asyncio.run(main_flow())

sys.exit(summary())