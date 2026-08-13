use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::{
    CommonResponse, ItemTypeAllResponse, ItemTypeListResponse, item_type::ItemTypeListRequest,
};

/// 列出某一层级的物品类型
/// 不递归遍历，只遍历子级
/// POST /item_type/get/list/{self}
#[utoipa::path(
    post,
    path = "/api/item_type/get/list/{self}",
    tag = "item-type",
    summary = "列出某一层级的物品类型",
    params(("self" = i64, Path, description = "是否仅自身（0/1）")),
    request_body = ItemTypeListRequest,
    responses(
        (status = 200, description = "类型分页列表", body = inline(CommonResponse<ItemTypeListResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn get_list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(self_flag): Path<i64>,
    AppJson(payload): AppJson<ItemTypeListRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match crate::functions::api::item_type::do_get_list(auth, self_flag != 0, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 列出所有物品类型
/// 不递归遍历，只遍历子级
/// POST /item_type/get/list_all
#[utoipa::path(
    post,
    path = "/api/item_type/get/list_all",
    tag = "item-type",
    summary = "列出所有物品类型",
    responses(
        (status = 200, description = "全部类型列表", body = inline(CommonResponse<ItemTypeAllResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn get_list_all(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, ApiError> {
    match crate::functions::api::item_type::do_get_list_all(auth).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
