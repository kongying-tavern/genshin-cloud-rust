use anyhow::Result;

use axum::{
    body::Bytes,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::middlewares::ExtractAuthInfo;
use axum::http::header;

/// 获取所有标签信息的压缩数据
/// GET /tag_doc/all_bin
#[tracing::instrument(skip(auth))]
pub async fn all_bin(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<Response, (StatusCode, String)> {
    match _functions::functions::api::tag_doc::do_all_bin(auth).await {
        Ok(bytes) => Ok((
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/octet-stream")],
            Bytes::from(bytes),
        )
            .into_response()),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}
