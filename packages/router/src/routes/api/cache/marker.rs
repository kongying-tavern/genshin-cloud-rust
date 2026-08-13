use anyhow::Result;

use crate::middlewares::{ApiError, ExtractAuthInfo};
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

/// 删除全部点位缓存
#[tracing::instrument(skip(auth))]
pub async fn delete_marker_cache(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::cache::do_delete_marker_cache(auth).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
