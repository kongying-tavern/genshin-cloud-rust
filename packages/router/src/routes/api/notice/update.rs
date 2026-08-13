use _utils::models::CommonResponse;
use anyhow::Result;

use axum::{extract::Json, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::notice::NoticeUpdateRequest;

/// 更新公告
#[utoipa::path(
    post,
    path = "/api/notice/update",
    tag = "notice",
    summary = "更新公告",
    request_body = NoticeUpdateRequest,
    responses(
        (status = 200, description = "更新结果", body = inline(CommonResponse<utoipa::TupleUnit>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn update_notice(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(request): AppJson<NoticeUpdateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::notice::do_update_notice(auth, request).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
