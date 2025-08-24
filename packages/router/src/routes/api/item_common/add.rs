use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 新增地区公用物品
/// 通过ID列表批量添加地区公用物品
/// PUT /item_common/add
#[tracing::instrument(skip_all)]
pub async fn add(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<Vec<i64>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现批量新增地区公用物品的逻辑
    Ok(())
}
