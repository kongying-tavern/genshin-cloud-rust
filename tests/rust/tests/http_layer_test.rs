//! HTTP-layer oneshot test — the first router-level test in the repo.
//!
//! Drives the Router assembled by `_router::routes::router()` directly via
//! `tower::ServiceExt::oneshot`: no port, no DB. Every case short-circuits in
//! the middleware / entry layer (401 / 403 / 413 / 501 / JWKS), so none of
//! them depend on a live database — route mounting, the auth middleware
//! chain, the #135 security response headers, the 16 MiB body limit, the
//! unconfigured-CORS path, and the #134 WS handshake auth contract are all
//! exercised without touching handler-internal business paths.
//!
//! Two planned cases had to be adapted to how axum 0.8 actually behaves
//! (found by reading the route table and the axum source, not assumed):
//!
//! - `POST /api/area/add` is actually a **PUT** route
//!   (`packages/router/src/routes/api/area/mod.rs`); a POST there would be
//!   rejected with 405 by method routing before the auth layer ever runs.
//! - The >16 MiB 413 case targets `/system/user/register/qq` (public, `Json`
//!   extractor) instead of `/oauth/token`: the oauth handler consumes its
//!   body via `Option<Multipart>` and maps read failures to a custom 400, so
//!   the router-level `DefaultBodyLimit` rejection (413) can never surface
//!   there. Both routes sit under the same router-level limit layer.

use std::net::SocketAddr;

use axum::{
    Router,
    body::{Body, to_bytes},
    extract::ConnectInfo,
    http::{HeaderMap, Request, StatusCode, header},
};
use tower::ServiceExt;

/// Test JWT secret: set at the start of every test (idempotent, same value).
/// ≥32 chars so it would also satisfy the startup strength check if that
/// ever applies to tests.
const TEST_JWT_SECRET: &str = "http-layer-test-secret-0123456789abcdef";

/// Common setup for every test in this binary: install the process-level
/// rustls CryptoProvider (the same call `main.rs` makes before building
/// anything — `router()` constructs reqwest clients for the CDN proxy at
/// assembly time, and with the workspace's provider-less rustls build they
/// panic without a process default, "No provider set"), then set the
/// process-wide JWT secret before anything touches the lazy key material
/// (`_utils::jwt::JWT_KEYS` panics without it).
fn setup() {
    // Idempotent at the process level: install_default() errs when a
    // provider is already installed (a parallel test may have won the race)
    // — that's success for our purposes.
    let _ = rustls::crypto::ring::default_provider().install_default();
    // SAFETY: every test in this binary sets the SAME value before any JWT
    // operation, so whichever test initializes the process-wide lazy keys
    // first, the captured value is deterministic (edition 2024 marks
    // set_var unsafe because concurrent readers could race).
    unsafe { std::env::set_var("JWT_SECRET", TEST_JWT_SECRET) };
}

/// Build a fresh Router. Route assembly is pure table building (no DB, no
/// network); each test builds its own because `oneshot` consumes the service.
async fn router() -> Router {
    _router::routes::router()
        .await
        .expect("failed to assemble router")
}

/// Inject the `ConnectInfo` extension a real listener would provide
/// (`into_make_service_with_connect_info` in `main.rs`). Handlers whose first
/// extractor is `ConnectInfo<SocketAddr>` (oauth, register_qq) need it.
fn with_connect_info(req: Request<Body>) -> Request<Body> {
    let (mut parts, body) = req.into_parts();
    parts
        .extensions
        .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 38000))));
    Request::from_parts(parts, body)
}

/// 1. Write endpoint without a token → 401 (auth middleware short-circuits).
///
/// NOTE: `/api/area/add` is mounted as PUT — see the module docs.
#[tokio::test]
async fn write_endpoint_without_token_is_401() {
    setup();
    let app = router().await;

    let resp = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/area/add")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("{}"))
                .expect("build request"),
        )
        .await
        .expect("call router");

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // The 401 body is the R-wrapped CommonResponse contract (real status
    // code so the frontend can trigger logout, JSON body for the message).
    let body = to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("read body");
    let v: serde_json::Value = serde_json::from_slice(&body).expect("body is JSON");
    assert_eq!(v["error"], serde_json::Value::Bool(true));
    assert_eq!(v["errorStatus"], serde_json::json!(401));
    assert_eq!(v["message"], "No Authorization header found");
}

/// 2. Garbage bearer token → 401, with the fixed message that does not echo
/// internal verification errors (algorithm/signature details).
#[tokio::test]
async fn write_endpoint_with_garbage_token_is_401() {
    setup();
    let app = router().await;

    let resp = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/area/add")
                .header(header::AUTHORIZATION, "Bearer garbage")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("{}"))
                .expect("build request"),
        )
        .await
        .expect("call router");

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let body = to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("read body");
    let v: serde_json::Value = serde_json::from_slice(&body).expect("body is JSON");
    assert_eq!(v["error"], serde_json::Value::Bool(true));
    assert_eq!(v["message"], "Invalid or expired token");
}

/// 3. JWKS is a public endpoint: 200 with a JSON `keys` array, and the #135
/// security response headers (`X-Content-Type-Options: nosniff`,
/// `Referrer-Policy: no-referrer`) are present on every response.
#[tokio::test]
async fn jwks_is_public_with_security_headers() {
    setup(); // jwks() touches the lazy key material even in HS256 mode
    let app = router().await;

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/.well-known/jwks.json")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("call router");

    assert_eq!(resp.status(), StatusCode::OK);

    // #135: overriding (per-response) security headers.
    assert_eq!(
        resp.headers()
            .get(header::X_CONTENT_TYPE_OPTIONS)
            .expect("x-content-type-options present"),
        "nosniff"
    );
    assert_eq!(
        resp.headers()
            .get(header::REFERRER_POLICY)
            .expect("referrer-policy present"),
        "no-referrer"
    );

    // HS256 mode (no RSA key configured) publishes an EMPTY key set — the
    // array must exist, the HMAC secret must not leak.
    let body = to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("read body");
    let v: serde_json::Value = serde_json::from_slice(&body).expect("body is JSON");
    let keys = v
        .get("keys")
        .and_then(|k| k.as_array())
        .expect("body has a keys array");
    assert!(keys.is_empty(), "HS256 mode must not publish key material");
}

/// 4. Body above the 16 MiB `DefaultBodyLimit` → 413 Payload Too Large.
///
/// Target is the public `/system/user/register/qq` (JSON extractor) — see
/// the module docs for why `/oauth/token` cannot surface this rejection.
#[tokio::test]
async fn oversized_body_is_413() {
    setup();
    let app = router().await;

    let oversized = vec![b'x'; 17 * 1024 * 1024]; // 17 MiB > 16 MiB limit
    let req = Request::builder()
        .method("POST")
        .uri("/system/user/register/qq")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(oversized))
        .expect("build request");

    let resp = app
        .oneshot(with_connect_info(req))
        .await
        .expect("call router");

    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

/// 5. Unconfigured CORS sends no `Access-Control-Allow-Origin`: without
/// `CORS_ALLOW_ORIGIN` the browser's default same-origin policy applies.
///
/// Only the unconfigured path is tested: the env var is process-global and
/// tokio tests in one binary run in parallel, so a configured-origin case
/// would race every other test that builds a Router (they would read a
/// half-configured CORS layer).
#[tokio::test]
async fn unconfigured_cors_sends_no_allow_origin_header() {
    // SAFETY: no test in this binary ever sets CORS_ALLOW_ORIGIN; removing
    // it only makes "unset" deterministic even when the ambient environment
    // happens to carry one.
    unsafe { std::env::remove_var("CORS_ALLOW_ORIGIN") };
    setup();
    let app = router().await;

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/.well-known/jwks.json")
                .header(header::ORIGIN, "https://example.com")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("call router");

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        resp.headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none(),
        "no CORS headers may be sent when CORS_ALLOW_ORIGIN is unset"
    );
}

/// 6. WS handshake auth contract (#134): missing token → 401, garbage
/// token → 401.
///
/// The 401/403 decisions live in `ws_handshake_key`, asserted directly
/// here: a oneshot request has no real connection upgrade
/// (`hyper::upgrade::OnUpgrade` has no public constructor), so axum's
/// `WebSocketUpgrade` extractor rejects any non-upgrade request before the
/// handler — and its auth — ever runs. A router-level call is kept as a
/// mounting smoke check (400 from the WS extractor, not 404).
#[tokio::test]
async fn ws_handshake_auth_rejects_missing_and_garbage_tokens() {
    setup();

    // Missing token (neither Authorization header nor ?token=).
    let err = _router::routes::ws::ws_handshake_key(&HeaderMap::new(), None, "123")
        .await
        .expect_err("handshake without a token must be rejected");
    assert_eq!(err, StatusCode::UNAUTHORIZED);

    // Garbage query token (?token=garbage — the browser-side channel).
    let err =
        _router::routes::ws::ws_handshake_key(&HeaderMap::new(), Some("garbage".into()), "123")
            .await
            .expect_err("handshake with a garbage token must be rejected");
    assert_eq!(err, StatusCode::UNAUTHORIZED);

    // Mounting smoke check: the route exists (not 404); the 400 is axum's
    // WS-extractor rejection for a request without upgrade headers.
    let app = router().await;
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/ws/123")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("call router");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

/// 7. Unknown path → the explicit fallback: 501 Not Implemented.
#[tokio::test]
async fn unknown_path_hits_501_fallback() {
    setup();
    let app = router().await;

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/nonexistent")
                .body(Body::empty())
                .expect("build request"),
        )
        .await
        .expect("call router");

    assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
    let body = to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("read body");
    assert_eq!(&body[..], &b"Not Implemented"[..]);
}
