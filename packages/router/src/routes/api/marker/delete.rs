use anyhow::Result;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 删除点位
/// DELETE /marker/{markerId}
#[tracing::instrument(skip_all)]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(marker_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现删除点位的逻辑
    Ok(())
}
