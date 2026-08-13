use crate::middlewares::{ApiError, ExtractAuthInfo};
use anyhow::Result;

use axum::{
    body::Bytes,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};

/// GET /marker_doc/list_diff_snapshot
#[utoipa::path(
    get,
    path = "/api/marker_doc/list_diff_snapshot",
    tag = "marker-doc",
    summary = "点位差异快照（GZIP 压缩二进制）",
    responses(
        (status = 200, description = "GZIP 压缩二进制", content_type = "application/octet-stream", body = Vec<u8>),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
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
