use _utils::models::CommonResponse;
use anyhow::Result;

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

/// 新增地区公用物品
/// 通过ID列表批量添加地区公用物品
/// PUT /item_common/add
#[utoipa::path(
    put,
    path = "/api/item_common/add",
    tag = "item-common",
    summary = "批量添加地区公用物品",
    request_body = Vec<i64>,
    responses(
        (status = 200, description = "添加结果", body = inline(CommonResponse<bool>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn add(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<Vec<i64>>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::item_common::do_add(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(serde_json::json!(v)))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
