use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::item_type::ItemTypeListRequest;

/// 列出某一层级的物品类型
/// 不递归遍历，只遍历子级
/// POST /item_type/get/list/{self}
#[tracing::instrument(skip_all)]
pub async fn get_list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(self_flag): Path<bool>,
    Json(payload): Json<ItemTypeListRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现物品类型列表查询逻辑
    Ok(())
}

/// 列出所有物品类型
/// 不递归遍历，只遍历子级
/// POST /item_type/get/list_all
#[tracing::instrument(skip_all)]
pub async fn get_list_all(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现获取所有物品类型的逻辑
    Ok(())
}
