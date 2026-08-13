use crate::middlewares::{ApiError, ExtractAuthInfo};
use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use _utils::models::{CommonResponse, common::EmptyResponse};

/// 删除地区公用物品
/// DELETE /item_common/delete/{itemId}
#[utoipa::path(
    delete,
    path = "/api/item_common/delete/{item_id}",
    tag = "item-common",
    summary = "删除地区公用物品",
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
    match crate::functions::api::item_common::do_delete(auth, item_id).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
