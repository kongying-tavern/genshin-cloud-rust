use anyhow::Result;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::middlewares::ExtractAuthInfo;
use axum::http::header;

/// 全量点位导出（GZIP 的 `MarkerVoList` protobuf；Java `listMarkersByBinary`）
/// GET /marker_doc/list_markers
///
/// 前端（map_register_v3 alova + 静态 protobuf 解码器）以 `arrayBuffer` 读取
/// 响应体 → `Zip.decompress`（GZIP）→ `protobuf.MarkerVoList.decode`，因此
/// Content-Type 用 octet-stream 且禁止缓存。
#[tracing::instrument(skip(db, auth))]
pub async fn list_markers(
    State(db): State<crate::routes::SharedDb>,
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<Response, crate::routes::RouteError> {
    match _functions::functions::api::marker_doc::do_list_markers(db.as_ref(), auth).await {
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
