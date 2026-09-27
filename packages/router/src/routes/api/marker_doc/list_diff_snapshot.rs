use anyhow::Result;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::middlewares::ExtractAuthInfo;
use axum::http::header;

/// 点位差异比对快照（未压缩 protobuf，`MarkerDiffSnapshotVoList`）
/// GET /marker_doc/list_diff_snapshot
#[tracing::instrument(skip(db, auth))]
pub async fn list_diff_snapshot(
    State(db): State<crate::routes::SharedDb>,
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<Response, crate::routes::RouteError> {
    match _functions::functions::api::marker_doc::do_list_diff_snapshot(db.as_ref(), auth).await {
        // Bytes 直接进响应体：缓存命中时全链路零拷贝（无整包 memcpy）。
        Ok(bytes) => Ok((
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "application/octet-stream"),
                (header::CACHE_CONTROL, "no-store"),
            ],
            bytes,
        )
            .into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
