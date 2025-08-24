use anyhow::Result;

use axum::{http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 物品分页的md5数组
/// GET /item_doc/list_page_bin_md5
#[tracing::instrument(skip(auth))]
pub async fn list_page_bin_md5(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现获取物品分页md5数组的逻辑
    Ok(())
}
