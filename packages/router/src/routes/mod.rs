mod api;
mod cdn;
mod system;
// `ws` 公开给 HTTP 层测试：oneshot 无法走真实 WS 升级，握手鉴权契约
// （`ws_handshake_key`）只能在函数层直测。
pub mod ws;

use anyhow::Result;

use axum::{
    Json, Router,
    extract::DefaultBodyLimit,
    http::StatusCode,
    http::header::{HeaderValue, REFERRER_POLICY, X_CONTENT_TYPE_OPTIONS},
    middleware::from_extractor,
    response::IntoResponse,
    routing::{get, post},
};

use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::TimeoutLayer;

use _utils::models::CommonResponse;

/// Handler 统一错误响应类型：HTTP 200 + R 包装（Java `RestException` 契约）。
///
/// Java 侧 `@RestControllerAdvice` 对 `GenshinApiException` 与兜底 `Throwable`
/// 都返回 HTTP 200 的 `R{errorStatus, message, data}`；前端 axios 只从 2xx 响应
/// 体读取 `data.error`/`data.message` 展示业务文案（非 2xx 走 axios 错误分支，
/// 只能拿到 "Request failed with status code xxx"）。因此这里保持 200 + JSON，
/// 鉴权失败（401/403）不走本类型，仍返回真实状态码以触发前端登出。
pub type RouteError = (StatusCode, Json<CommonResponse<()>>);

/// 业务错误的 R 包装：HTTP 200 + `{error:true, errorStatus:500, message}`。
fn route_error(message: impl Into<String>) -> RouteError {
    (
        StatusCode::OK,
        Json(
            CommonResponse::<()>::new(Err(anyhow::anyhow!(message.into())))
                .with_status(StatusCode::INTERNAL_SERVER_ERROR.as_u16()),
        ),
    )
}

/// 携带真实 HTTP 状态码的 R 包装错误（用于 401/403 等需要前端按状态码
/// 触发登出的分支；正文仍是 JSON R 结构）。
pub fn status_error(code: u16, message: impl Into<String>) -> RouteError {
    let status = StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (
        status,
        Json(
            CommonResponse::<()>::new(Err(anyhow::anyhow!(message.into())))
                .with_status(status.as_u16()),
        ),
    )
}

/// 将 handler 返回的错误映射为统一 R 包装：
/// - 业务错误（如 "Item not found"，由 `anyhow!("...")` 产生）保留原文返回给客户端；
/// - SQL/DB/Redis/MinIO/IO 等内部错误只记日志，向客户端返回通用消息（Java 的
///   「请求失败」），避免泄露内部细节。
pub fn internal_error<E: Into<anyhow::Error>>(e: E) -> RouteError {
    let err: anyhow::Error = e.into();
    let detail = format!("{err}");
    let chain = err
        .chain()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(" | ")
        .to_lowercase();
    const INTERNAL_KEYWORDS: [&str; 10] = [
        "db",
        "sql",
        "connection",
        "connect",
        "database",
        "redis",
        "minio",
        "s3",
        "bucket",
        "os error",
    ];
    if INTERNAL_KEYWORDS.iter().any(|kw| chain.contains(kw)) {
        tracing::error!("internal server error: {detail}");
        return route_error("请求失败");
    }
    route_error(detail)
}

/// 请求级超时上限。60s：挡住慢请求长期占用 handler 的面（慢 SQL、挂起
/// 的上游调用），同时给 `POST /api/score/data`（`ExtractAdmin` 鉴权、
/// limit 上限 100_000 的重查询）留足余量；CDN 代理内部另有更短的 15s
/// 上游超时，不会先撞到这里。
const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/oauth/token", post(system::oauth::oauth))
        .route("/.well-known/jwks.json", get(jwks))
        .nest("/system", system::router().await?)
        .route("/ws/{user_id}", get(ws::ws_handler))
        // The domain endpoints live under /api/* (Java contract — the
        // frontend's production build and direct clients call these paths).
        // They are ALSO merged at the root: the Vite dev proxy rewrites
        // `/api/*` → `/*` before forwarding (vite.config `rewrite`), so a
        // dev-mode frontend hits the unprefixed paths. Both must work.
        .merge(api::router().await?)
        .nest("/api", api::router().await?)
        .nest_service("/cdn", cdn::cdn_proxy())
        .fallback(|| async { (StatusCode::NOT_IMPLEMENTED, "Not Implemented").into_response() })
        .layer(cors_layer())
        .layer(from_extractor::<crate::middlewares::ExtractUserAgent>())
        .layer(from_extractor::<crate::middlewares::ExtractIP>())
        .layer(DefaultBodyLimit::max(1024 * 1024 * 16)) // 16 MiB
        // 请求级 60s 超时（理由见 REQUEST_TIMEOUT），超时以 408 拒绝
        // （tower-http 0.7 已弃用默认状态码的 `TimeoutLayer::new`）。
        // 注意 tower-http 的 Timeout 只约束「响应 future」（收到请求 →
        // 响应头就绪）：WebSocket 的 101 握手响应立即返回，升级完成后的
        // 长连 socket 不在覆盖范围，不会被 60s 掐断——勿因「WS 看似不受
        // 限」而误删本层。
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            REQUEST_TIMEOUT,
        ))
        // 安全响应头（overriding 保证逐响应存在）：
        // - nosniff：禁止浏览器对响应体做 MIME 嗅探——/cdn 图片代理硬编码
        //   `Content-Type: image/png`（不随真实文件类型变化），该路径尤其
        //   需要这层兜底；
        // - Referrer-Policy: no-referrer：不向第三方泄漏站内 URL。
        // HSTS 留给 TLS 终结层（本服务纯 HTTP，在此声明反而误导）。
        .layer(SetResponseHeaderLayer::overriding(
            X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            REFERRER_POLICY,
            HeaderValue::from_static("no-referrer"),
        ));

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
