use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use _utils::models::marker_link::MarkerLinkage;
use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};

/// 关联点位
/// POST /marker_link/link
#[tracing::instrument(skip(auth))]
pub async fn link(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<Vec<MarkerLinkage>>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::marker_link::do_link(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
