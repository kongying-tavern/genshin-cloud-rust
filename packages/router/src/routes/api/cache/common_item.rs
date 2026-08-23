use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAdmin;

/// 删除全部公用物品缓存
#[tracing::instrument(skip(auth))]
pub async fn delete_common_item_cache(
    ExtractAdmin(auth): ExtractAdmin,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::cache::do_delete_common_item_cache(auth).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
