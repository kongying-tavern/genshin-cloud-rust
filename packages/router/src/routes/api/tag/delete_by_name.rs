use anyhow::Result;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractManager;

/// 按标签名软删除标签（前端兼容路由）
/// DELETE /tag/{tagName}
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractManager(auth): ExtractManager,
    Path(tag_name): Path<String>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::tag::do_delete_by_name(auth, tag_name).await {
        Ok(resp) => Ok((StatusCode::OK, axum::Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
