use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::icon::IconUpdateRequest;

/// 修改图标信息
/// 由icon_id定位修改一个icon
/// POST /icon/update
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<IconUpdateRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
