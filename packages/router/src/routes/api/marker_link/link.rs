use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::marker_link::MarkerLinkage;

/// 关联点位
/// POST /marker_link/link
#[tracing::instrument(skip(auth))]
pub async fn link(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<Vec<MarkerLinkage>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现关联点位的逻辑
    Ok(())
}
