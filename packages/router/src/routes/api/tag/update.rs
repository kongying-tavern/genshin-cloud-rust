use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use _utils::models::TagUpdateRequest;
use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};

/// 更新标签
/// POST /tag/update
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<TagUpdateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::tag::do_update(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
