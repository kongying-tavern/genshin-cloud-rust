use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::TagTypeListRequest;

/// 标签类型列表
/// POST /tag_type/get/list
#[tracing::instrument(skip(auth))]
pub async fn list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<TagTypeListRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::tag_type::do_list(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
