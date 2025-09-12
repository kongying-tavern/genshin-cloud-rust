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
#[tracing::instrument(skip(auth))]
pub async fn get_list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(self_flag): Path<bool>,
    Json(payload): Json<ItemTypeListRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match crate::functions::api::item_type::do_get_list(auth, self_flag, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}

/// 列出所有物品类型
/// 不递归遍历，只遍历子级
/// POST /item_type/get/list_all
#[tracing::instrument(skip(auth))]
pub async fn get_list_all(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match crate::functions::api::item_type::do_get_list_all(auth).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
