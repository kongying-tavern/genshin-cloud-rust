use anyhow::Result;

use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::middlewares::ExtractAuthInfo;

/// 删除标签缓存
/// DELETE /cache/icon_tag
#[tracing::instrument(skip(_auth))]
pub async fn delete_icon_tag_cache(
    ExtractAuthInfo(_auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::cache::do_delete_icon_tag_cache(_auth).await {
        Ok(_) => Ok((StatusCode::OK, axum::Json(serde_json::json!({})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}
