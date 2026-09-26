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

use _utils::errors::DomainError;
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

/// 将 handler 返回的错误映射为统一 R 包装，按**三层判定**（顺序固定）：
///
/// 1. **领域错误**：沿 `err.chain()` 逐层 `downcast_ref::<DomainError>()`
///    （业务层显式构造的分类，见 `_utils::errors`），命中后按变体映射：
///    `Business(msg)` → `route_error(msg)`（HTTP 200 + errorStatus 500 +
///    原文案，Java `RestException` 契约）；`BadRequest(msg)` → 真实 400 +
///    R 包装；`Forbidden(msg)` → 真实 403 + R 包装。
/// 2. **内部错误类型**：链上任一层可 downcast 为 `sea_orm::DbErr` /
///    `redis::RedisError` / `reqwest::Error` / `std::io::Error` /
///    `minio::s3::error::Error` —— 记完整错误链日志，向客户端返回通用
///    「请求失败」（Java 同文案），不泄露 SQL / 连接 / 存储等内部细节。
/// 3. **兜底**：既非领域错误也非内部类型（未迁移到 `DomainError` 的
///    角落）→ 保持旧行为，`route_error(原文)` 返回最外层文案。
///
/// 注意：anyhow 的 `downcast_ref` 只查**最外层**；`context(...)` 包裹后
/// 领域/内部类型位于链的内层，必须沿 `chain()` 逐层 downcast。
pub fn internal_error<E: Into<anyhow::Error>>(e: E) -> RouteError {
    let err: anyhow::Error = e.into();
    for cause in err.chain() {
        if let Some(domain) = cause.downcast_ref::<DomainError>() {
            return match domain {
                DomainError::Business(msg) => route_error(msg.clone()),
                DomainError::BadRequest(msg) => {
                    status_error(StatusCode::BAD_REQUEST.as_u16(), msg.clone())
                },
                DomainError::Forbidden(msg) => {
                    status_error(StatusCode::FORBIDDEN.as_u16(), msg.clone())
                },
            };
        }
    }
    if chain_contains_internal(&err) {
        let detail = err
            .chain()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(" | ");
        tracing::error!("internal server error: {detail}");
        return route_error("请求失败");
    }
    route_error(err.to_string())
}

/// 内部错误类型判定（清单与语义见 [`internal_error`] 文档第 2 层）。
/// 这些错误经 `?` 直接传播进 anyhow 时类型保留在链上，可精确识别；
/// 业务层已 `map_err` 压成文案的例外见各调用点注释。
fn chain_contains_internal(err: &anyhow::Error) -> bool {
    err.chain().any(|cause| {
        cause.downcast_ref::<sea_orm::DbErr>().is_some()
            || cause.downcast_ref::<redis::RedisError>().is_some()
            || cause.downcast_ref::<reqwest::Error>().is_some()
            || cause.downcast_ref::<std::io::Error>().is_some()
            || cause.downcast_ref::<minio::s3::error::Error>().is_some()
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 解包 RouteError，便于对状态码与 R 包装体分别断言。
    fn unwrap(resp: RouteError) -> (StatusCode, CommonResponse<()>) {
        let (status, Json(body)) = resp;
        (status, body)
    }

    /// 第 1 层（Business）：HTTP 200 + errorStatus 500 + 原文案 —— Java
    /// `RestException` 契约，前端 axios 从 2xx 响应体读取业务文案。
    #[test]
    fn business_error_maps_to_200_with_original_message() {
        let (status, body) = unwrap(internal_error(DomainError::Business(
            "该点位已更新，请重新提交".into(),
        )));
        assert_eq!(status, StatusCode::OK);
        assert!(body.error);
        assert_eq!(body.error_status, 500);
        assert_eq!(body.message, "该点位已更新，请重新提交");
    }

    /// 第 1 层必须沿 chain 逐层 downcast：`context(...)` 包裹后领域类型在
    /// 链的内层，且响应须用领域文案而非外层 context 文案（anyhow 的
    /// `downcast_ref` 只查最外层，这正是要逐层遍历的原因）。
    #[test]
    fn business_error_downcasts_through_context() {
        let err = anyhow::Error::new(DomainError::Business("username exists".into()))
            .context("register failed");
        let (status, body) = unwrap(internal_error(err));
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.error_status, 500);
        assert_eq!(body.message, "username exists");
    }

    /// 第 1 层（BadRequest）：真实 400 + R 包装（如旧密码错误）。
    #[test]
    fn bad_request_error_maps_to_real_400() {
        let (status, body) = unwrap(internal_error(DomainError::BadRequest(
            "Old password is incorrect".into(),
        )));
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(body.error);
        assert_eq!(body.error_status, 400);
        assert_eq!(body.message, "Old password is incorrect");
    }

    /// 第 1 层（Forbidden）：真实 403 + R 包装。
    #[test]
    fn forbidden_error_maps_to_real_403() {
        let (status, body) = unwrap(internal_error(DomainError::Forbidden(
            "Forbidden: only admins can update other users".into(),
        )));
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert!(body.error);
        assert_eq!(body.error_status, 403);
        assert_eq!(
            body.message,
            "Forbidden: only admins can update other users"
        );
    }

    /// 第 2 层（内部类型）：`DbErr`（连接故障）即使被 context 包裹也要命中
    /// —— 只记日志（此处不断言日志），向客户端返回通用「请求失败」，
    /// 不泄露 SQL / 连接细节。选 `Conn(RuntimeErr::Internal)` 而非
    /// `RecordNotFound`：后者语义偏业务（找不到记录），前者是真正的
    /// 基础设施故障。
    #[test]
    fn db_err_is_masked_as_generic_failure() {
        let err = anyhow::Error::new(sea_orm::DbErr::Conn(sea_orm::RuntimeErr::Internal(
            "connection refused".into(),
        )))
        .context("query users");
        let (status, body) = unwrap(internal_error(err));
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.error_status, 500);
        assert_eq!(body.message, "请求失败");
    }

    /// 第 3 层（兜底）：未迁移的 `anyhow!("文案")` 保持旧行为 ——
    /// HTTP 200 + errorStatus 500 + 原文返回，避免未迁移角落行为回归。
    #[test]
    fn plain_anyhow_error_falls_back_to_original_text() {
        let (status, body) = unwrap(internal_error(anyhow::anyhow!("未迁移的角落文案")));
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.error_status, 500);
        assert_eq!(body.message, "未迁移的角落文案");
    }

    /// 403 统一形状（role_extrator::authorize / admin_extrator 的 403 分支
    /// 即本构造）：真实 403 + JSON R 包装，与 auth_extrator 的 401 形状
    /// 对齐。router 级 oneshot 无法覆盖该分支——合法签名 token 会在
    /// `oauth_parse_token` 的会话校验处阻塞等待 DB（无库测试环境挂起），
    /// 故以本单测锁定形状契约。
    #[test]
    fn forbidden_403_shape_is_json_r_wrapped() {
        let (status, body) = unwrap(status_error(StatusCode::FORBIDDEN.as_u16(), "Forbidden"));
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert!(body.error);
        assert_eq!(body.error_status, 403);
        assert_eq!(body.message, "Forbidden");
    }
}
