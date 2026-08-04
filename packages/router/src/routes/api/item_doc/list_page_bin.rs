use anyhow::Result;

use axum::{
    body::Bytes,
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::middlewares::ExtractAuthInfo;
use axum::http::header;

/// 物品分页数据（GZIP 压缩二进制）
/// GET /item_doc/list_page_bin/{md5}
#[tracing::instrument(skip(auth))]
pub async fn list_page_bin(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(md5): Path<String>,
) -> Result<Response, (StatusCode, String)> {
    match _functions::functions::api::item_doc::do_list_page_bin(auth, md5).await {
        Ok(bytes) => Ok((
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "application/octet-stream"),
                (header::CACHE_CONTROL, "no-store"),
            ],
            Bytes::from(bytes),
        )
            .into_response()),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}
