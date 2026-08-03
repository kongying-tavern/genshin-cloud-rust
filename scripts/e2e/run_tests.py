#!/usr/bin/env python3
"""E2E browser tests driven by Shirabe headless browser.

Starts a Shirabe debug server, then drives the browser to verify the Vue3
frontend + Rust backend stack end-to-end.

Usage:
    python scripts/e2e/run_tests.py
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import time
from pathlib import Path

import urllib.request

sys.path.insert(0, str(Path(__file__).resolve().parent))
from config import RUST_URL, SHIRABE_PORT, SHIRABE_URL, STATE_DIR, VUE_URL  # noqa: E402

SCREENSHOTS_DIR = STATE_DIR / "screenshots"
SCREENSHOTS_DIR.mkdir(parents=True, exist_ok=True)


# ── Shirabe HTTP client ──────────────────────────────────────────────────────


def _shirabe_post(path: str, data: dict, timeout: float = 30) -> dict:
    """POST to the Shirabe debug API."""
    url = f"{SHIRABE_URL}{path}"
    body = json.dumps(data).encode("utf-8")
    req = urllib.request.Request(
        url, data=body, headers={"Content-Type": "application/json"}
    )
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return json.loads(resp.read().decode("utf-8"))


def _shirabe_get(path: str, timeout: float = 30) -> dict:
    """GET from the Shirabe debug API."""
    url = f"{SHIRABE_URL}{path}"
    with urllib.request.urlopen(url, timeout=timeout) as resp:
        return json.loads(resp.read().decode("utf-8"))


def _wait_for_shirabe(timeout: float = 30) -> bool:
    """Wait for the Shirabe debug server to be ready."""
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            _shirabe_get("/health", timeout=3)
            return True
        except Exception:
            time.sleep(1)
    return False


# ── Test cases ───────────────────────────────────────────────────────────────

TESTS_PASSED = 0
TESTS_FAILED = 0
TESTS_SKIPPED = 0


class SkipTest(Exception):
    """Raised when a test cannot run in the current environment (e.g. no
    credentials configured). Reported as SKIP — never as PASS or FAIL."""


def test(name: str, fn):
    """Run a test function, reporting pass/fail/skip."""
    global TESTS_PASSED, TESTS_FAILED, TESTS_SKIPPED
    print(f"\n{'─' * 60}")
    print(f"TEST: {name}")
    try:
        fn()
        TESTS_PASSED += 1
        print(f"  ✅ PASS")
    except SkipTest as e:
        TESTS_SKIPPED += 1
        print(f"  ⏭️  SKIP: {e}")
    except Exception as e:
        TESTS_FAILED += 1
        print(f"  ❌ FAIL: {e}")


def _api_headers() -> dict:
    """Authenticated headers for API calls.

    When E2E_USERNAME / E2E_PASSWORD are set, logs in via the password grant
    and returns an Authorization bearer header. Otherwise returns an empty
    header set — the API tests then SKIP instead of treating 401/403 as a
    pass.
    """
    import os

    username = os.environ.get("E2E_USERNAME")
    password = os.environ.get("E2E_PASSWORD")
    if not username or not password:
        raise SkipTest(
            "no E2E_USERNAME / E2E_PASSWORD configured — set them to run "
            "authenticated API assertions"
        )
    body, content_type = _multipart_form({
        "grant_type": "password",
        "username": username,
        "password": password,
    })
    req = urllib.request.Request(
        f"{RUST_URL}/oauth/token",
        data=body,
        headers={"Content-Type": content_type},
    )
    with urllib.request.urlopen(req, timeout=15) as resp:
        data = json.loads(resp.read().decode("utf-8"))
    token = data.get("access_token")
    if not token:
        raise SkipTest(f"login succeeded but no access_token in response: {data}")
    return {"Authorization": f"Bearer {token}"}


def _multipart_form(fields: dict) -> tuple[bytes, str]:
    """Encode plain-text form fields as multipart/form-data.

    The Rust `/oauth/token` endpoint extracts the password grant via axum's
    `Multipart` (Java-parity), so urlencoded bodies are rejected with a 400 —
    the client must send multipart with a boundary.
    """
    import uuid

    boundary = uuid.uuid4().hex
    chunks: list[bytes] = []
    for key, value in fields.items():
        chunks.append(f"--{boundary}".encode())
        chunks.append(
            f'Content-Disposition: form-data; name="{key}"'.encode("utf-8")
        )
        chunks.append(b"")
        chunks.append(str(value).encode("utf-8"))
    chunks.append(f"--{boundary}--".encode())
    return b"\r\n".join(chunks), f"multipart/form-data; boundary={boundary}"


def test_page_loads():
    """Verify the Vue frontend page loads."""
    _shirabe_post("/navigate", {"url": VUE_URL})
    time.sleep(5)  # let SPA render

    # Take screenshot
    resp = _shirabe_post("/screenshot", {})
    # Shirabe returns {"ok": true, "data": {"data": "<base64>", "mime_type": "image/png", ...}}
    screenshot_b64 = None
    if isinstance(resp, dict):
        data = resp.get("data", resp)
        if isinstance(data, dict):
            screenshot_b64 = data.get("data")
        elif isinstance(data, str):
            screenshot_b64 = data

    if screenshot_b64:
        import base64

        img_path = SCREENSHOTS_DIR / "01_page_load.png"
        img_path.write_bytes(base64.b64decode(screenshot_b64))
        print(f"  📸 Screenshot: {img_path}")
    else:
        print(f"  ⚠️  No screenshot data")

    # Check page title via evaluate
    try:
        result = _shirabe_post("/evaluate", {"expression": "document.title"})
        title = ""
        if isinstance(result, dict):
            data = result.get("data", result)
            if isinstance(data, dict):
                title = str(data.get("result", ""))
            elif isinstance(data, str):
                title = data
        print(f"  Page title: {title}")
    except Exception as e:
        print(f"  ⚠️  Could not read title: {e}")


def test_api_area_list():
    """Verify the Rust backend serves area data through the Vite proxy.

    Requires configured credentials (E2E_USERNAME / E2E_PASSWORD): without
    them the test SKIPs — a 401/403 is no longer treated as a pass.
    """
    import urllib.error
    import urllib.request as ur

    headers = _api_headers()
    headers["Content-Type"] = "application/json"
    url = f"{VUE_URL}/api/area/get/list"
    req = ur.Request(url, method="POST", data=b"{}", headers=headers)
    try:
        with ur.urlopen(req, timeout=10) as resp:
            body = resp.read().decode("utf-8")
            print(f"  Response: {body[:200]}...")
            payload = json.loads(body)
            if "data" not in payload and "items" not in payload:
                raise Exception(f"unexpected response shape: {body[:200]}")
    except urllib.error.HTTPError as e:
        if e.code in (401, 403):
            raise SkipTest(f"endpoint requires auth (HTTP {e.code}) — "
                           "check E2E_USERNAME / E2E_PASSWORD")
        raise Exception(f"HTTP {e.code}: {e.read().decode('utf-8', errors='replace')[:200]}")
    except Exception as e:
        raise Exception(f"API call failed: {e}")


def test_api_marker_doc_md5():
    """Verify the BinaryMD5 marker doc endpoint returns the MD5 list."""
    import urllib.request as ur

    headers = _api_headers()
    url = f"{VUE_URL}/api/marker_doc/list_page_bin_md5"
    req = ur.Request(url, method="GET", headers=headers)
    try:
        with ur.urlopen(req, timeout=15) as resp:
            body = resp.read().decode("utf-8")
            print(f"  Response: {body[:200]}...")
            payload = json.loads(body)
            data = payload.get("data") if isinstance(payload, dict) else None
            if not isinstance(data, list):
                raise Exception(f"expected a data list, got: {body[:200]}")
    except urllib.error.HTTPError as e:
        if e.code in (401, 403):
            raise SkipTest(f"endpoint requires auth (HTTP {e.code}) — "
                           "check E2E_USERNAME / E2E_PASSWORD")
        raise Exception(f"HTTP {e.code}: {e.read().decode('utf-8', errors='replace')[:200]}")


def test_api_item_doc_md5():
    """Verify the BinaryMD5 item doc endpoint returns the MD5 list."""
    import urllib.request as ur

    headers = _api_headers()
    url = f"{VUE_URL}/api/item_doc/list_page_bin_md5"
    req = ur.Request(url, method="GET", headers=headers)
    try:
        with ur.urlopen(req, timeout=15) as resp:
            body = resp.read().decode("utf-8")
            print(f"  Response: {body[:200]}...")
            payload = json.loads(body)
            data = payload.get("data") if isinstance(payload, dict) else None
            if not isinstance(data, list):
                raise Exception(f"expected a data list, got: {body[:200]}")
    except urllib.error.HTTPError as e:
        if e.code in (401, 403):
            raise SkipTest(f"endpoint requires auth (HTTP {e.code}) — "
                           "check E2E_USERNAME / E2E_PASSWORD")
        raise Exception(f"HTTP {e.code}: {e.read().decode('utf-8', errors='replace')[:200]}")


def test_rust_health():
    """Verify the Rust backend is responding directly."""
    import urllib.error
    import urllib.request as ur

    try:
        with ur.urlopen(f"{RUST_URL}/", timeout=5) as resp:
            print(f"  Rust backend HTTP {resp.status}")
    except urllib.error.HTTPError as e:
        # 404, 401, 501 etc. — server IS responding
        print(f"  Rust backend HTTP {e.code} (server alive)")
    except Exception as e:
        if "ConnectionResetError" in str(type(e).__name__):
            print(f"  (Server listening — connection reset expected)")
        else:
            raise


# ── Main ─────────────────────────────────────────────────────────────────────


def start_shirabe() -> subprocess.Popen | None:
    """Start the Shirabe debug server."""
    print(f"🌐 Starting Shirabe debug server on port {SHIRABE_PORT}...")
    env = {**os.environ, "SHIRABE_PORT": str(SHIRABE_PORT)}
    proc = subprocess.Popen(
        ["npx", "@celestia-island/shirabe", "debug", "--port", str(SHIRABE_PORT)],
        env=env,
        stdout=open(STATE_DIR / "shirabe.log", "w", encoding="utf-8"),
        stderr=subprocess.STDOUT,
        shell=True,
    )

    if _wait_for_shirabe(timeout=30):
        print(f"✅ Shirabe ready at {SHIRABE_URL}")
        return proc
    else:
        print(f"❌ Shirabe did not start in 30s", file=sys.stderr)
        proc.kill()
        return None


def main() -> int:
    # Start Shirabe
    shirabe_proc = start_shirabe()
    if shirabe_proc is None:
        return 1

    try:
        # Run tests
        test("Rust backend health", test_rust_health)
        test("Vue page loads", test_page_loads)
        test("API: area list (proxy)", test_api_area_list)
        test("API: marker doc MD5", test_api_marker_doc_md5)
        test("API: item doc MD5", test_api_item_doc_md5)

        # Summary
        print(f"\n{'═' * 60}")
        print(f"RESULTS: {TESTS_PASSED} passed, {TESTS_FAILED} failed, {TESTS_SKIPPED} skipped")
        print(f"Screenshots: {SCREENSHOTS_DIR}")
        return 0 if TESTS_FAILED == 0 else 1
    finally:
        shirabe_proc.terminate()
        try:
            shirabe_proc.wait(timeout=5)
        except Exception:
            shirabe_proc.kill()


if __name__ == "__main__":
    raise SystemExit(main())
