use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::types::SystemUserRole;

/// 返回可用角色列表
/// GET /role/list
#[tracing::instrument(skip(auth))]
pub async fn list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if auth.info.role_id != SystemUserRole::Admin {
        return Ok((axum::http::StatusCode::FORBIDDEN, "Forbidden".to_string()).into_response());
    }

    Ok(Json(()).into_response())
}
