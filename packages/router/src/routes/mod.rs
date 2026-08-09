mod api;
mod system;

use anyhow::Result;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::StatusCode,
    middleware::from_extractor,
    response::IntoResponse,
    routing::{get, post},
};

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/oauth/token", post(system::oauth::oauth))
        .route("/oauth/qq", post(system::oauth::qq_login))
        .route("/.well-known/jwks.json", get(jwks))
        .nest("/system", system::router().await?)
        // The domain endpoints live under /api/* (Java contract — the
        // frontend's production build and direct clients call these paths).
        // They are ALSO merged at the root: the Vite dev proxy rewrites
        // `/api/*` → `/*` before forwarding (vite.config `rewrite`), so a
        // dev-mode frontend hits the unprefixed paths. Both must work.
        .merge(api::router().await?)
        .nest("/api", api::router().await?)
        .nest_service("/cdn", cdn_proxy())
        .fallback(|| async { (StatusCode::NOT_IMPLEMENTED, "Not Implemented").into_response() })
        .layer(cors_layer())
        .layer(from_extractor::<crate::middlewares::ExtractUserAgent>())
        .layer(from_extractor::<crate::middlewares::ExtractIP>())
        .layer(DefaultBodyLimit::max(1024 * 1024 * 16)); // 16 MiB

    Ok(ret)
}

/// 可配置的 CORS 策略：
/// - `CORS_ALLOW_ORIGIN` 设置允许的源（逗号分隔），未设置时**不允许**任何
///   跨域请求（浏览器前端应走同源/Vite 代理）；
/// - 允许的源会收到 `Authorization` 头放行与标准的 GET/POST/PUT/DELETE。
fn cors_layer() -> tower_http::cors::CorsLayer {
    use axum::http::header::HeaderValue;
    use tower_http::cors::{AllowOrigin, CorsLayer};

    let allowed = std::env::var("CORS_ALLOW_ORIGIN")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .map(|v| {
            v.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        });

    let base = CorsLayer::new()
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
        ]);

    match allowed {
        Some(origins) if !origins.is_empty() => {
            let origins: Vec<HeaderValue> = origins.iter().filter_map(|o| o.parse().ok()).collect();
            base.allow_origin(AllowOrigin::list(origins))
        },
        // 未配置白名单：不发送任何 CORS 头，浏览器默认阻止跨域读取。
        _ => base,
    }
}

/// JWKS 公钥分发端点（`GET /.well-known/jwks.json`），无鉴权。
async fn jwks() -> axum::response::Response {
    match _functions::functions::system::oauth::do_jwks().await {
        Ok(v) => axum::Json(v).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("JWKS generation failed: {e}"),
        )
            .into_response(),
    }
}

/// CDN 上游地址：默认 `v3.yuanshen.site`，可用 `CDN_UPSTREAM` 环境变量覆盖
/// （例如自建 CDN 或内网镜像）。URL 不带尾斜杠。
fn cdn_upstream() -> String {
    std::env::var("CDN_UPSTREAM")
        .unwrap_or_else(|_| "https://v3.yuanshen.site".into())
        .trim_end_matches('/')
        .to_string()
}

/// 本地 dadian 配置文件路径（`CDN_DADIAN_CONFIG`，可选）。
/// 指向一个预生成的 bz2 压缩配置；未设置时 `/cdn/dadian-preview.json.bz2`
/// 降级为内置的空配置（开发期行为）。
fn dadian_config_file() -> Option<Vec<u8>> {
    let path = std::env::var("CDN_DADIAN_CONFIG").ok()?;
    std::fs::read(path).ok()
}

/// CDN proxy with fallback. Tries remote CDN first, falls back to a locally
/// generated minimal dadian config if the remote file is unavailable.
fn cdn_proxy() -> axum::Router {
    use axum::extract::State;
    use axum::http::Request;
    use axum::response::{IntoResponse, Response};
    use std::time::Duration;

    // 上游挂起时不能无限等待：连接 5s、整体 15s 超时。
    let client = std::sync::Arc::new(
        reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(15))
            .build()
            .expect("build cdn client"),
    );
    let upstream = cdn_upstream();
    let dadian_override = dadian_config_file();

    Router::new()
        .fallback(
            move |State(client): State<std::sync::Arc<reqwest::Client>>,
                  req: Request<axum::body::Body>| {
                let client = client.clone();
                let upstream = upstream.clone();
                let dadian_override = dadian_override.clone();
                async move {
                    let path = req.uri().path().trim_start_matches("/cdn");

                    // Special case: serve locally generated dadian config
                    if path == "/dadian-preview.json.bz2" {
                        let bz2_bytes = match dadian_override {
                            Some(bytes) => bytes,
                            None => {
                                let config = serde_json::json!({
                                    "tiles": {},
                                    "application": {"avatar": [], "nameCard": []},
                                    "editor": {},
                                    "plugins": {}
                                });
                                let json_bytes = serde_json::to_vec(&config).unwrap_or_default();
                                bz2_bump(&json_bytes)
                            },
                        };
                        return Response::builder()
                            .status(200)
                            .header("Content-Type", "application/octet-stream")
                            .header("Access-Control-Allow-Origin", "*")
                            .body(axum::body::Body::from(bz2_bytes))
                            .unwrap_or_else(|_| {
                                Response::builder()
                                    .status(500)
                                    .body(axum::body::Body::from("compress error"))
                                    .unwrap()
                            })
                            .into_response();
                    }

                    // Image proxy: forward to the icon host with wildcard CORS
                    // (the dev MinIO on ddns.minemc.top sends no CORS headers,
                    // which breaks the frontend sprite renderer).
                    if let Some(query) = req.uri().query()
                        && let Some(u) = query.split('&').find_map(|kv| kv.strip_prefix("u="))
                    {
                        let decoded = urlencoding::decode(u).unwrap_or_default().into_owned();
                        match client.get(&decoded).send().await {
                            Ok(resp) => {
                                let body = resp.bytes().await.unwrap_or_default();
                                return Response::builder()
                                    .status(200)
                                    .header("Content-Type", "image/png")
                                    .header("Access-Control-Allow-Origin", "*")
                                    .header("Cache-Control", "no-store")
                                    .body(axum::body::Body::from(body))
                                    .unwrap()
                                    .into_response();
                            },
                            Err(_) => {
                                return Response::builder()
                                    .status(502)
                                    .header("Access-Control-Allow-Origin", "*")
                                    .body(axum::body::Body::from("img proxy error"))
                                    .unwrap()
                                    .into_response();
                            },
                        }
                    }
                    // All other CDN paths: proxy to the configured upstream
                    let url = format!("{upstream}{path}");

                    match client.get(&url).send().await {
                        Ok(resp) => {
                            let status = resp.status();
                            let headers = resp.headers().clone();
                            let body = resp.bytes().await.unwrap_or_default();

                            // Check if CDN returned HTML (SPA fallback) instead of real content
                            let is_html =
                                body.starts_with(b"<!DOCTYPE") || body.starts_with(b"<html");
                            if is_html {
                                return Response::builder()
                                    .status(404)
                                    .header("Access-Control-Allow-Origin", "*")
                                    .body(axum::body::Body::from("not found"))
                                    .unwrap()
                                    .into_response();
                            }

                            let mut builder = Response::builder().status(status);
                            for (k, v) in headers.iter() {
                                if k != "transfer-encoding" && k != "content-length" {
                                    builder = builder.header(k, v);
                                }
                            }
                            builder = builder.header("Access-Control-Allow-Origin", "*");
                            // 上游响应无缓存头：浏览器可能启发式缓存到之前
                            // 网络抖动期的坏响应（502/空 body），禁止缓存。
                            builder = builder.header("Cache-Control", "no-store");
                            builder
                                .body(axum::body::Body::from(body))
                                .unwrap_or_else(|_| {
                                    Response::builder()
                                        .status(500)
                                        .body(axum::body::Body::from("proxy error"))
                                        .unwrap()
                                })
                                .into_response()
                        },
                        Err(e) => Response::builder()
                            .status(502)
                            .header("Access-Control-Allow-Origin", "*")
                            .header("Cache-Control", "no-store")
                            .body(axum::body::Body::from(format!("CDN proxy error: {e}")))
                            .unwrap()
                            .into_response(),
                    }
                }
            },
        )
        .with_state(client)
}

/// Compress data with bz2 (pure Rust, no C dependency).
fn bz2_bump(data: &[u8]) -> Vec<u8> {
    use std::io::Write;
    let mut writer = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::default());
    writer.write_all(data).ok();
    writer.finish().unwrap_or_default()
}
