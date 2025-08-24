use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::item_type::ItemTypeUpdateData;

/// 修改物品类型
/// POST /item_type/update
#[tracing::instrument(skip_all)]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<ItemTypeUpdateData>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现修改物品类型的逻辑
    Ok(())
}
