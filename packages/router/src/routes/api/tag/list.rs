use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::tag::TagListRequest;

/// 列出标签
/// 可按照分类（分类需保证为末端分类）进行查询，也可给出需要查询url的tag名称列表，可分页
/// POST /tag/get/list
#[tracing::instrument(skip(auth))]
pub async fn list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<TagListRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
