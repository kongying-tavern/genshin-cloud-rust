use anyhow::Result;

use axum::{
    body::Bytes,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::middlewares::ExtractAuthInfo;
use axum::http::header;

/// 点位差异比对快照（未压缩 protobuf，`MarkerDiffSnapshotVoList`）
/// GET /marker_doc/list_diff_snapshot
#[tracing::instrument(skip(auth))]
pub async fn list_diff_snapshot(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<Response, crate::routes::RouteError> {
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
