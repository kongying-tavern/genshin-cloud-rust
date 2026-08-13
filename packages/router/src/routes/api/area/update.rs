use anyhow::Result;

use axum::{extract::Json, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::{AreaUpdateRequest, common::EmptyResponse, wrapper::CommonResponse};

/// 修改地区
/// POST /area/update
#[utoipa::path(
    post,
    path = "/api/area/update",
    tag = "area",
    summary = "修改地区",
    request_body = AreaUpdateRequest,
    responses(
        (status = 200, description = "更新结果", body = inline(CommonResponse<EmptyResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<AreaUpdateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::area::do_update(auth, payload).await {
        Ok(_) => Ok(Json(CommonResponse::new(Ok(EmptyResponse {}))).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
