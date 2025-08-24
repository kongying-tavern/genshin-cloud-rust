use anyhow::Result;

use axum::{http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 删除全部点位缓存
#[tracing::instrument(skip(auth))]
pub async fn delete_marker_cache(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现点位缓存删除逻辑
    Ok(())
}
