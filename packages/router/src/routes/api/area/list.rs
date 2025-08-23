use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};
use _utils::models::AreaListRequest;

use crate::middlewares::ExtractAuthInfo;

/// 列出地区
/// POST /area/get/list
/// 可根据父级地区id列出子地区列表
#[tracing::instrument(skip_all)]
pub async fn list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<AreaListRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
