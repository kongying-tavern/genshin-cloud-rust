use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 删除标签缓存
#[tracing::instrument(skip(auth))]
pub async fn delete_icon_tag_cache(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(request): Json<Vec<String>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::cache::do_delete_icon_tag_cache(auth).await {
        Ok(_) => Ok((StatusCode::OK, Json(serde_json::json!({})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
