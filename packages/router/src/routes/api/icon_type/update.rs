use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::icon_type::IconTypeUpdateRequest;

/// 修改分类
/// 由类型ID来定位修改一个分类
/// POST /icon_type/update
#[tracing::instrument(skip_all)]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<IconTypeUpdateRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
