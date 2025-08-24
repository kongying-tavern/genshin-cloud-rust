use anyhow::Result;

use axum::{extract::Multipart, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 上传图片
#[tracing::instrument(skip_all)]
pub async fn upload_image(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现图片上传逻辑
    Ok(())
}
