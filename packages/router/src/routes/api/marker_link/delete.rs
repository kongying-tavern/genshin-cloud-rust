use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::marker_link::MarkerLinkDeleteRequest;

/// 删除点位关联
/// DELETE /marker_link/delete
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<MarkerLinkDeleteRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现删除点位关联的逻辑
    Ok(())
}
