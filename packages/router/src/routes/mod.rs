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
        .route("/.well-known/jwks.json", get(jwks))
        .nest("/system", system::router().await?)
        .merge(api::router().await?)
        .nest_service("/cdn", cdn_proxy())
        .fallback(|| async { (StatusCode::NOT_IMPLEMENTED, "Not Implemented").into_response() })
        .layer(from_extractor::<crate::middlewares::ExtractUserAgent>())
        .layer(from_extractor::<crate::middlewares::ExtractIP>())
        .layer(DefaultBodyLimit::max(1024 * 1024 * 16)); // 16 MiB

    Ok(ret)
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

/// CDN proxy with fallback. Tries remote CDN first, falls back to a locally
/// generated minimal dadian config if the remote file is unavailable.
fn cdn_proxy() -> axum::Router {
    use axum::extract::State;
    use axum::http::Request;
    use axum::response::{IntoResponse, Response};

    let client = std::sync::Arc::new(reqwest::Client::new());

    Router::new()
        .fallback(
            move |State(client): State<std::sync::Arc<reqwest::Client>>,
                  req: Request<axum::body::Body>| {
                let client = client.clone();
                async move {
                    let path = req.uri().path().trim_start_matches("/cdn");

                    // Special case: serve locally generated dadian config
                    if path == "/dadian-preview.json.bz2" {
                        let config = serde_json::json!({
                            "tiles": {},
                            "application": {"avatar": [], "nameCard": []},
                            "editor": {},
                            "plugins": {}
                        });
                        let json_bytes = serde_json::to_vec(&config).unwrap_or_default();
                        let bz2_bytes = bz2_bump(&json_bytes);
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

                    // All other CDN paths: proxy to v3.yuanshen.site
                    let url = format!("https://v3.yuanshen.site{}", path);

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
