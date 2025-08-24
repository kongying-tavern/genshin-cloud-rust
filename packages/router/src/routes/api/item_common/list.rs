use anyhow::Result;
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::Pagination;

/// 列出地区公用物品
/// 列出公共物品，但需要注意处理所属地区已被删除的公共物品
/// POST /item_common/get/list
#[tracing::instrument(skip_all)]
pub async fn get_list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<Pagination>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现地区公用物品列表查询逻辑
    // 需要注意处理所属地区已被删除的公共物品
    Ok(())
}
