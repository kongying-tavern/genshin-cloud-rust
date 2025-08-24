use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::icon::IconListRequest;

/// 列出图标
/// 可按照分类（分类需保证为末端分类）和上传者进行查询，也可根据ID批量查询，可分页
/// POST /icon/get/list
#[tracing::instrument(skip(auth))]
pub async fn list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<IconListRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
