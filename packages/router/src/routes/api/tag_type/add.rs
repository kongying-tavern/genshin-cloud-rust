use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::TagTypeAddRequest;

/// 新增标签类型
/// PUT /tag_type/add
#[tracing::instrument(skip(auth))]
pub async fn add(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<TagTypeAddRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::tag_type::do_add(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
