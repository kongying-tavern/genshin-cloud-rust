use anyhow::Result;

use axum::{extract::Json, extract::Path, response::IntoResponse};

use crate::middlewares::{ApiError, ExtractAuthInfo};
use _utils::models::{common::EmptyResponse, wrapper::CommonResponse};

/// 删除物品类型
/// 批量递归删除物品类型，需在前端做二次确认
/// DELETE /item_type/delete/{itemTypeId}
#[utoipa::path(
    delete,
    path = "/api/item_type/delete/{item_type_id}",
    tag = "item-type",
    summary = "删除物品类型（递归删除）",
    params(("item_type_id" = i64, Path, description = "物品类型 ID")),
    responses(
        (status = 200, description = "删除结果", body = inline(CommonResponse<EmptyResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(item_type_id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::item_type::do_delete(auth, item_type_id).await {
        Ok(_) => Ok(Json(CommonResponse::new(Ok(EmptyResponse {}))).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
