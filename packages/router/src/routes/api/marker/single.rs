use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::marker::{MarkerAddRequest, MarkerUpdateData};

/// 新增点位
/// PUT /marker/single
#[tracing::instrument(skip_all)]
pub async fn add_single(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<MarkerAddRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现新增点位的逻辑
    Ok(())
}

/// 修改点位
/// POST /marker/single
#[tracing::instrument(skip_all)]
pub async fn update_single(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<MarkerUpdateData>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现修改点位的逻辑
    Ok(())
}
