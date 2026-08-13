use anyhow::Result;

use axum::{
    body::Bytes,
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::middlewares::{ApiError, ExtractAuthInfo};
use axum::http::header;

/// 点位分页数据（GZIP 压缩二进制）
/// GET /marker_doc/list_page_bin/{md5}
#[utoipa::path(
    get,
    path = "/api/marker_doc/list_page_bin/{md5}",
    tag = "marker-doc",
    summary = "点位分页数据（GZIP 压缩二进制）",
    params(("md5" = String, Path, description = "页数据 MD5")),
    responses(
        (status = 200, description = "GZIP 压缩二进制", content_type = "application/octet-stream", body = Vec<u8>),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn list_page_bin(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(md5): Path<String>,
) -> Result<Response, ApiError> {
    match _functions::functions::api::marker_doc::do_list_page_bin(auth, md5).await {
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
