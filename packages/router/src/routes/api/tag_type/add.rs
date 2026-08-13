use _utils::models::CommonResponse;
use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::TagTypeAddRequest;

/// 新增标签类型
/// PUT /tag_type/add
#[utoipa::path(
    put,
    path = "/api/tag_type/add",
    tag = "tag-type",
    summary = "新增标签类型",
    request_body = TagTypeAddRequest,
    responses(
        (status = 200, description = "新类型 ID", body = inline(CommonResponse<i64>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn add(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<TagTypeAddRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::tag_type::do_add(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
