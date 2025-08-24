use anyhow::Result;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 物品分页数据
/// GET /item_doc/list_page_bin/{md5}
#[tracing::instrument(skip(auth))]
pub async fn list_page_bin(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(md5): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现根据md5获取物品分页数据的逻辑
    Ok(())
}
