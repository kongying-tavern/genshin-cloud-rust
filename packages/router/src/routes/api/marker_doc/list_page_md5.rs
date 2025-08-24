use anyhow::Result;
use axum::{http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 点位分页的md5数组
/// 返回点位分页bz2的md5数组
/// GET /marker_doc/list_page_bin_md5
#[tracing::instrument(skip_all)]
pub async fn list_page_bin_md5(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现获取点位分页md5数组的逻辑
    Ok(())
}
