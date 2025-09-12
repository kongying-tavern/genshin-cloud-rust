use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 删除全部物品缓存
#[tracing::instrument(skip(auth))]
pub async fn delete_item_cache(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::cache::do_delete_item_cache(auth).await {
        Ok(_) => Ok((StatusCode::OK, Json(serde_json::json!({})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
