use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::TagAddRequest;

/// 新增标签
/// PUT /tag/add
/// 返回新增标签 ID
#[tracing::instrument(skip(auth))]
pub async fn add(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<TagAddRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::tag::do_add(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(serde_json::json!({"id": resp.id})))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
