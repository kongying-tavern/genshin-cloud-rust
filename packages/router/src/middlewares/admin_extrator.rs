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
            return Err((StatusCode::FORBIDDEN, "Forbidden".to_string()).into_response());
        }
        Ok(Self(auth))
    }
}
