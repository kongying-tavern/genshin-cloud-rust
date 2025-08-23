use anyhow::Result;

use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::tag_type::TagTypeListRequest;

/// 列出分类
/// 列出标签的分类，parentID为-1的时候为列出所有的根分类，可分页
/// POST /tag_type/get/list
#[tracing::instrument(skip_all)]
pub async fn list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<TagTypeListRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
