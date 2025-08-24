use anyhow::Result;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 删除地区公用物品
/// DELETE /item_common/delete/{itemId}
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(item_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现删除地区公用物品的逻辑
    Ok(())
}
