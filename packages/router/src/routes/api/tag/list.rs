use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use _utils::models::TagListRequest;
use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};

/// 标签列表
/// POST /tag/get/list
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
