use anyhow::Result;

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use _utils::models::{CommonResponse, ItemVO};

/// 根据物品ID查询物品
/// 输入ID列表查询，单个查询也用此API
/// POST /item/get/list_by_id
#[utoipa::path(
    post,
    path = "/api/item/get/list_byid",
    tag = "item",
    summary = "根据物品 ID 查询物品",
    request_body = Vec<i64>,
    responses(
        (status = 200, description = "物品列表", body = inline(CommonResponse<Vec<ItemVO>>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn get_list_by_id(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<Vec<i64>>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::item::do_get_list_by_id(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
