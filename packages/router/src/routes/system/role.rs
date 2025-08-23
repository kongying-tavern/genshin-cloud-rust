use anyhow::Result;

use axum::{http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 返回可用角色列表
/// GET /role/list
#[tracing::instrument(skip_all)]
pub async fn list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
