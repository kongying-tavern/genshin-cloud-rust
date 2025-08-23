use anyhow::Result;

use axum::{
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;

/// 所有标签信息的压缩
/// 查询所有标签信息，返回bz2压缩格式的byte数组
/// GET /tag_doc/all_bin
#[tracing::instrument(skip_all)]
pub async fn all_bin(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
