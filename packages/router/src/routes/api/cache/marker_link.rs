use anyhow::Result;

use axum::{http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 删除全部点位关联缓存
#[tracing::instrument(skip(auth))]
pub async fn delete_marker_link_cache(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现点位关联缓存删除逻辑
    Ok(())
}
