use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};

use _utils::{jwt::AuthInfo, types::SystemUserRole};

use super::auth_extrator::ExtractAuthInfo;

/// Admin-only auth extractor: authenticates like `ExtractAuthInfo` and rejects
/// any non-Admin role with 403. Replaces the repetitive
/// `if auth.info.role_id != SystemUserRole::Admin { 403 }` boilerplate in the
/// system routes.
pub struct ExtractAdmin(pub AuthInfo);

impl<S> FromRequestParts<S> for ExtractAdmin
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let ExtractAuthInfo(auth) = ExtractAuthInfo::from_request_parts(parts, state).await?;
        if auth.info.role_id != SystemUserRole::Admin {
            // 403 与 auth_extrator 的 401 形状对齐：JSON R 包装（而非纯文本），
            // 前端可用同一套解析读取 errorStatus/message。
            return Err(
                crate::routes::status_error(StatusCode::FORBIDDEN.as_u16(), "Forbidden")
                    .into_response(),
            );
        }
        Ok(Self(auth))
    }
}
