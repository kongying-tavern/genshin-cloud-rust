use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::TagTypeListRequest;
use _utils::models::{CommonResponse, TagTypeListResponse};

/// 标签类型列表
/// POST /tag_type/get/list
#[utoipa::path(
    post,
    path = "/api/tag_type/get/list",
    tag = "tag-type",
    summary = "标签类型列表",
    request_body = TagTypeListRequest,
    responses(
        (status = 200, description = "类型分页列表", body = inline(CommonResponse<TagTypeListResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<TagTypeListRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::tag_type::do_list(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
