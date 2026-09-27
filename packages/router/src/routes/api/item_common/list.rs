use anyhow::Result;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractManager;
use _utils::models::Pagination;

/// 列出地区公用物品
/// 列出公共物品，但需要注意处理所属地区已被删除的公共物品
/// POST /item_common/get/list
#[tracing::instrument(skip(db, auth))]
pub async fn get_list(
    State(db): State<crate::routes::SharedDb>,
    ExtractManager(auth): ExtractManager,
    Json(payload): Json<Pagination>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match crate::functions::api::item_common::do_get_list(db.as_ref(), auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
