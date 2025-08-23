use anyhow::Result;

use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::tag_type::TagTypeAddRequest;

/// 新增分类
/// 类型id在创建后返回
/// PUT /tag_type/add
#[tracing::instrument(skip_all)]
pub async fn add(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<TagTypeAddRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
