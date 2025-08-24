use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::item::ItemFilterRequest;

/// 根据筛选条件列出物品信息
/// 传入的物品类型ID和地区ID列表，必须为末端的类型或地区
/// POST /item/get/list
#[tracing::instrument(skip_all)]
pub async fn get_list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<ItemFilterRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现物品列表查询逻辑
    Ok(())
}
