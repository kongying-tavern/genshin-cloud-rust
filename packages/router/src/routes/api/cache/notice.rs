use anyhow::Result;

use axum::{http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 删除公告缓存
#[tracing::instrument(skip(auth))]
pub async fn delete_notice_cache(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现公告缓存删除逻辑
    Ok(())
}
