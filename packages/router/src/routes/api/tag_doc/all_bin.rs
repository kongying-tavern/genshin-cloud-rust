use anyhow::Result;

use axum::{
    body::Bytes,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::middlewares::ExtractAuthInfo;
use _database::DB_CONN;
use axum::http::header;

/// 获取所有标签信息的压缩数据
/// GET /tag_doc/all_bin
#[tracing::instrument(skip(auth))]
pub async fn all_bin(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<Response, crate::routes::RouteError> {
    match _functions::functions::api::tag_doc::do_all_bin(DB_CONN.wait().as_ref(), auth).await {
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
