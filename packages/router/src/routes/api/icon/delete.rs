use anyhow::Result;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 删除图标
/// DELETE /icon/delete/{iconId}
#[tracing::instrument(skip(_auth))]
pub async fn delete(
    ExtractAuthInfo(_auth): ExtractAuthInfo,
    Path(icon_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
