use anyhow::Result;

use axum::{http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 列出分类
/// 列出图标的分类，typeId为-1的时候为列出所有的根分类
/// POST /icon_type/get/list
#[tracing::instrument(skip(_auth))]
pub async fn list(
    ExtractAuthInfo(_auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
