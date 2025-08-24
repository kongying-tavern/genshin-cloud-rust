use anyhow::Result;

use axum::{http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 删除全部物品缓存
#[tracing::instrument(skip(auth))]
pub async fn delete_item_cache(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现物品缓存删除逻辑
    Ok(())
}
