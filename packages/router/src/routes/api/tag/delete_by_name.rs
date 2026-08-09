use anyhow::Result;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 按标签名软删除标签（前端兼容路由）
/// DELETE /tag/{tagName}
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(tag_name): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::tag::do_delete_by_name(auth, tag_name).await {
        Ok(resp) => Ok((StatusCode::OK, axum::Json(resp))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
