use anyhow::Result;
use axum::{http::StatusCode, response::IntoResponse};
use crate::middlewares::ExtractAuthInfo;

/// 删除地区缓存
#[tracing::instrument(skip_all)]
pub async fn delete_area_cache(ExtractAuthInfo(auth): ExtractAuthInfo) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现地区缓存删除逻辑
    Ok(())
}
