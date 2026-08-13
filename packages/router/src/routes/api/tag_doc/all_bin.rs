use anyhow::Result;

use axum::{
    body::Bytes,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::middlewares::{ApiError, ExtractAuthInfo};
use axum::http::header;

/// 获取所有标签信息的压缩数据
/// GET /tag_doc/all_bin
#[utoipa::path(
    get,
    path = "/api/tag_doc/all_bin",
    tag = "tag-doc",
    summary = "获取所有标签信息的压缩数据",
    responses(
        (status = 200, description = "GZIP 压缩二进制", content_type = "application/octet-stream", body = Vec<u8>),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn all_bin(ExtractAuthInfo(auth): ExtractAuthInfo) -> Result<Response, ApiError> {
    match _functions::functions::api::tag_doc::do_all_bin(auth).await {
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
