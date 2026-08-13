use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::notice::NoticeListRequest;
use _utils::models::{CommonResponse, notice::NoticeListResponse};

/// 获取公告列表
#[utoipa::path(
    post,
    path = "/api/notice/get/list",
    tag = "notice",
    summary = "获取公告列表",
    request_body = NoticeListRequest,
    responses(
        (status = 200, description = "公告分页列表", body = inline(CommonResponse<NoticeListResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn get_notice_list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(request): AppJson<NoticeListRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::notice::do_get_notice_list(auth, request).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
