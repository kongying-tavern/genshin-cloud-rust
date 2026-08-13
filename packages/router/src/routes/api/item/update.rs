use anyhow::Result;

use axum::{
    extract::{Json, Path},
    response::IntoResponse,
};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::{common::EmptyResponse, item::ItemUpdateData, wrapper::CommonResponse};

/// 修改物品
/// 提供修改同名物品功能，默认关闭
/// POST /item/update/{editSame}
#[utoipa::path(
    post,
    path = "/api/item/update/{edit_same}",
    tag = "item",
    summary = "修改物品",
    params(("edit_same" = i64, Path, description = "是否允许修改同名物品（0/1）")),
    request_body = Vec<ItemUpdateData>,
    responses(
        (status = 200, description = "更新结果", body = inline(CommonResponse<EmptyResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(edit_same): Path<i64>,
    AppJson(payload): AppJson<Vec<ItemUpdateData>>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::item::do_update(auth, edit_same != 0, payload).await {
        Ok(_) => Ok(Json(CommonResponse::new(Ok(EmptyResponse {}))).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
