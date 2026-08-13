use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::history::HistoryListRequest;
use _utils::models::{CommonResponse, history::HistoryListResponse};

/// 历史记录分页查询
/// POST /history/get/list
#[utoipa::path(
    post,
    path = "/api/history/get/list",
    tag = "history",
    summary = "历史记录分页查询",
    request_body = HistoryListRequest,
    responses(
        (status = 200, description = "历史记录分页列表", body = inline(CommonResponse<HistoryListResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn get_list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<HistoryListRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::history::do_get_list(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
