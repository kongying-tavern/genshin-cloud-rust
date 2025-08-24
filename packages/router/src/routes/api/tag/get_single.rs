use anyhow::Result;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 获取单个标签信息
/// POST /tag/get/single/{name}
#[tracing::instrument(skip(auth))]
pub async fn get_single(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
