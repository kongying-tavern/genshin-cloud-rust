use crate::middlewares::{ApiError, ExtractAuthInfo};
use anyhow::Result;

use axum::{
    body::Bytes,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};

/// GET /marker_doc/list_diff_snapshot
#[tracing::instrument(skip(auth))]
pub async fn list_diff_snapshot(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<Response, ApiError> {
    match _functions::functions::api::marker_doc::do_list_diff_snapshot(auth).await {
        Ok(bytes) => Ok((
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "application/octet-stream"),
                (header::CACHE_CONTROL, "no-store"),
            ],
            Bytes::from(bytes),
        )
            .into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
