use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAdmin;

/// 删除全部物品缓存
#[tracing::instrument(skip(auth))]
pub async fn delete_item_cache(
    ExtractAdmin(auth): ExtractAdmin,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::cache::do_delete_item_cache(auth).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
