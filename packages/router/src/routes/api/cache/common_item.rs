use anyhow::Result;

use axum::{http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 删除全部公用物品缓存
#[tracing::instrument(skip_all)]
pub async fn delete_common_item_cache(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现公用物品缓存删除逻辑
    Ok(())
}
