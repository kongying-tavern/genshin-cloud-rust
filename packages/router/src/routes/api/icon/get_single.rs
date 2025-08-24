use anyhow::Result;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 获取单个图标信息
/// POST /icon/get/single/{iconId}
#[tracing::instrument(skip(auth))]
pub async fn get_single(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(icon_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
