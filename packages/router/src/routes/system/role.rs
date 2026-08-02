use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAdmin;

/// 返回可用角色列表
/// GET /role/list
#[tracing::instrument(skip(_auth))]
pub async fn list(
    ExtractAdmin(_auth): ExtractAdmin,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(Json(()).into_response())
}
