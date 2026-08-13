use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::TagListRequest;
use _utils::models::{CommonResponse, TagListResponse};

/// 标签列表
/// POST /tag/get/list
#[utoipa::path(
    post,
    path = "/api/tag/get/list",
    tag = "tag",
    summary = "标签列表",
    request_body = TagListRequest,
    responses(
        (status = 200, description = "标签分页列表", body = inline(CommonResponse<TagListResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<TagListRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::tag::do_list(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
