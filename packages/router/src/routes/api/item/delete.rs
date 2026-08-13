use anyhow::Result;

use axum::{extract::Json, extract::Path, response::IntoResponse};

use crate::middlewares::{ApiError, ExtractAuthInfo};
use _utils::models::{common::EmptyResponse, wrapper::CommonResponse};

/// 删除物品
/// 根据物品ID删除物品
/// DELETE /item/delete/{itemId}
#[utoipa::path(
    delete,
    path = "/api/item/delete/{item_id}",
    tag = "item",
    summary = "删除物品",
    params(("item_id" = i64, Path, description = "物品 ID")),
    responses(
        (status = 200, description = "删除结果", body = inline(CommonResponse<EmptyResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(item_id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    match crate::functions::api::item::do_delete(auth, item_id).await {
        Ok(_) => Ok(Json(CommonResponse::new(Ok(EmptyResponse {}))).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
