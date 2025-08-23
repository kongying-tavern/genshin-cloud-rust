use anyhow::Result;

use _utils::models::AreaUpdateRequest;
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 修改地区
/// POST /area/update
#[tracing::instrument(skip_all)]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<AreaUpdateRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
