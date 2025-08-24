use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::item_type::ItemTypeAddRequest;

/// 添加物品类型
/// 成功后返回新的类型ID
/// PUT /item_type/add
#[tracing::instrument(skip(auth))]
pub async fn add(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<ItemTypeAddRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现新增物品类型的逻辑
    Ok(())
}
