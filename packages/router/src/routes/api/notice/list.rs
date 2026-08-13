use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use _utils::models::notice::NoticeListRequest;
use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};

/// 获取公告列表
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
