use anyhow::Result;

use crate::middlewares::{ApiError, ExtractAuthInfo};
use axum::{extract::Path, http::StatusCode, response::IntoResponse};

/// 按标签名软删除标签（前端兼容路由）
/// DELETE /tag/{tagName}
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(tag_name): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::tag::do_delete_by_name(auth, tag_name).await {
        Ok(resp) => Ok((StatusCode::OK, axum::Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
