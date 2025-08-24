use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 删除标签缓存
#[tracing::instrument(skip(auth))]
pub async fn delete_icon_tag_cache(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(request): Json<Vec<String>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现标签缓存删除逻辑
    Ok(())
}
