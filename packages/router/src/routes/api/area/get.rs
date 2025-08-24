use anyhow::Result;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 获取单个地区信息
/// POST /area/get/{areaId}
#[tracing::instrument(skip(auth))]
pub async fn get(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(area_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
