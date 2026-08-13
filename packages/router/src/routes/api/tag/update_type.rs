use _utils::models::CommonResponse;
use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::tag::TagUpdateTypeRequest;

/// 修改标签的分类信息（后台接口）
/// POST /tag/updateType
#[utoipa::path(
    post,
    path = "/api/tag/updateType",
    tag = "tag",
    summary = "修改标签的分类信息（后台接口）",
    request_body = TagUpdateTypeRequest,
    responses(
        (status = 200, description = "更新结果", body = inline(CommonResponse<bool>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth, payload))]
pub async fn update_type(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<TagUpdateTypeRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::tag::do_update_type(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
