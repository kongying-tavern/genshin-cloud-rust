use _utils::models::CommonResponse;
use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::item_type::ItemTypeAddRequest;

/// 添加物品类型
/// 成功后返回新的类型ID
/// PUT /item_type/add
#[utoipa::path(
    put,
    path = "/api/item_type/add",
    tag = "item-type",
    summary = "添加物品类型",
    request_body = ItemTypeAddRequest,
    responses(
        (status = 200, description = "新类型 ID", body = inline(CommonResponse<i64>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn add(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<ItemTypeAddRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::item_type::do_add(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
