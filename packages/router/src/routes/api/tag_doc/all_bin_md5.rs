use anyhow::Result;

use axum::{
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;

/// 所有标签信息的md5
/// GET /tag_doc/all_bin_md5
#[tracing::instrument(skip(auth))]
pub async fn all_bin_md5(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
