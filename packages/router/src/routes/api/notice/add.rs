use anyhow::Result;

use axum::{extract::Json, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::notice::NoticeAddRequest;

/// 新增公告
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
