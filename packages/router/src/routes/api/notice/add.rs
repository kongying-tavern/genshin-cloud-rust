use _utils::models::CommonResponse;
use anyhow::Result;

use axum::{extract::Json, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::notice::NoticeAddRequest;

/// 新增公告
#[utoipa::path(
    put,
    path = "/api/notice/add",
    tag = "notice",
    summary = "新增公告",
    request_body = NoticeAddRequest,
    responses(
        (status = 200, description = "新公告 ID", body = inline(CommonResponse<i64>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn add_notice(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(request): AppJson<NoticeAddRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::notice::do_add_notice(auth, request).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
