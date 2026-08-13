use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use _utils::models::marker_link::MarkerLinkDeleteRequest;
use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};

/// 删除点位关联
/// DELETE /marker_link/delete
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<MarkerLinkDeleteRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::marker_link::do_delete(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
