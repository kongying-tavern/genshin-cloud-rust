use anyhow::Result;

use crate::middlewares::{ApiError, ExtractAuthInfo};
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

/// 删除地区缓存
#[tracing::instrument(skip(auth))]
pub async fn delete_area_cache(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::cache::do_delete_area_cache(auth).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
