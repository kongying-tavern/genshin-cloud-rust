use anyhow::Result;

use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::tag_type::TagTypeUpdateRequest;

/// 修改分类
/// POST /tag_type/update
#[tracing::instrument(skip_all)]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<TagTypeUpdateRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
