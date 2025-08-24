use anyhow::Result;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 删除物品
/// 根据物品ID删除物品
/// DELETE /item/delete/{itemId}
#[tracing::instrument(skip_all)]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(item_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现删除物品的逻辑
    Ok(())
}
