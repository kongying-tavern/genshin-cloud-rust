use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use _utils::models::Pagination;
use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};

/// 列出地区公用物品
/// 列出公共物品，但需要注意处理所属地区已被删除的公共物品
/// POST /item_common/get/list
#[tracing::instrument(skip(auth))]
pub async fn get_list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<Pagination>,
) -> Result<impl IntoResponse, ApiError> {
    match crate::functions::api::item_common::do_get_list(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
