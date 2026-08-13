use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::TagTypeUpdateRequest;

/// 更新标签类型
/// POST /tag_type/update
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<TagTypeUpdateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::tag_type::do_update(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
