use anyhow::{Result, anyhow};

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};

use _database::DB_CONN;
use _functions::functions::system::oauth::oauth_parse_token;
use _utils::{jwt::AuthInfo, models::CommonResponse};

pub struct ExtractAuthInfo(pub AuthInfo);

impl<S> FromRequestParts<S> for ExtractAuthInfo
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Ok(bearer) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state).await
        {
            let token = bearer.token().to_string();
            // 组合根取全局连接用 get() 而非 wait()：wait() 在全局未初始化时
            // 永久阻塞，且参数在进入 oauth_parse_token 前就求值——无 DB 的
            // HTTP 层测试（只到中间件层）会当场卡死。未初始化 = 无法认证，
            // 与无效令牌同口径 401；生产环境 main.rs 启动即 init_db_conn，
            // 正常流量不会走到该分支。
            let Some(db) = DB_CONN.get().map(|m| m.as_ref()) else {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    serde_json::to_string(&CommonResponse::<()>::new(Err(anyhow!(
                        "Invalid or expired token"
                    ))))
                    .expect("Failed to serialize error response"),
                )
                    .into_response());
            };
            // 401 固定文案：不回显 verify_token 的内部错误（算法/签名细节）
            let (info, claims) = oauth_parse_token(db, token).await.map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    serde_json::to_string(&CommonResponse::<()>::new(Err(anyhow!(
                        "Invalid or expired token"
                    ))))
                    .expect("Failed to serialize error response"),
                )
                    .into_response()
            })?;

            return Ok(Self(AuthInfo {
                info,
                created_at: claims.iat,
                expires_at: claims.exp,
            }));
        }

        let ret = (
            StatusCode::UNAUTHORIZED,
            serde_json::to_string(
                &CommonResponse::<()>::new(Err(anyhow!("No Authorization header found")))
                    .with_status(StatusCode::UNAUTHORIZED.as_u16()),
            )
            .expect("Failed to serialize error response"),
        );
        Err(ret.into_response())
    }
}
