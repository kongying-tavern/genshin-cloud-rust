use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::TagTypeUpdateRequest;
use _utils::models::{CommonResponse, common::EmptyResponse};

/// 更新标签类型
/// POST /tag_type/update
#[utoipa::path(
    post,
    path = "/api/tag_type/update",
    tag = "tag-type",
    summary = "更新标签类型",
    request_body = TagTypeUpdateRequest,
    responses(
        (status = 200, description = "更新结果", body = inline(CommonResponse<EmptyResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<TagTypeUpdateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::tag_type::do_update(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
