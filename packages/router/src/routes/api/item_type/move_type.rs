use anyhow::Result;

use axum::{
    extract::{Json, Path},
    response::IntoResponse,
};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::{common::EmptyResponse, wrapper::CommonResponse};

/// 批量移动类型为目标类型的子类型
/// 将类型批量移动到某个类型下作为其子类型
/// POST /item_type/move/{targetTypeId}
#[utoipa::path(
    post,
    path = "/api/item_type/move/{target_type_id}",
    tag = "item-type",
    summary = "批量移动类型为目标类型的子类型",
    params(("target_type_id" = i64, Path, description = "目标类型 ID")),
    request_body = Vec<i64>,
    responses(
        (status = 200, description = "移动结果", body = inline(CommonResponse<EmptyResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn move_to_target(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(target_type_id): Path<i64>,
    AppJson(payload): AppJson<Vec<i64>>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::item_type::do_move_to_target(auth, target_type_id, payload)
        .await
    {
        Ok(_) => Ok(Json(CommonResponse::new(Ok(EmptyResponse {}))).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
