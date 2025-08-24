use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::AreaListRequest;

/// 列出地区
/// POST /area/get/list
/// 可根据父级地区id列出子地区列表
#[tracing::instrument(skip(auth))]
pub async fn list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<AreaListRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
