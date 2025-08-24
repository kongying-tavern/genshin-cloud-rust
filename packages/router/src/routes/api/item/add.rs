use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::item::ItemAddRequest;

/// 新增物品
/// 新建成功后会返回新物品ID
/// PUT /item/add
#[tracing::instrument(skip(auth))]
pub async fn add(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<ItemAddRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现新增物品的逻辑
    Ok(())
}
