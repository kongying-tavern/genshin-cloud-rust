use anyhow::Result;

use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;

/// 删除标签
/// 需要确保已经没有条目在使用这个标签，否则会删除失败
/// DELETE /tag/{tagName}
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(tag_name): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
