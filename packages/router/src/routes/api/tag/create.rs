use anyhow::Result;

use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;

/// 创建标签
/// 只创建一个空标签
/// PUT /tag/{tagName}
#[tracing::instrument(skip(auth))]
pub async fn create(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(tag_name): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
