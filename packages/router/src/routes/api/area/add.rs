use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::AreaAddRequest;

/// 新增地区
/// PUT /area/add
/// 返回新增地区ID
#[tracing::instrument(skip(auth))]
pub async fn add(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<AreaAddRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
