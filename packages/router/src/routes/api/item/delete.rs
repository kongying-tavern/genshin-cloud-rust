use anyhow::Result;

use axum::{extract::Json, extract::Path, response::IntoResponse};

use crate::middlewares::ExtractManager;
use _utils::models::{common::EmptyResponse, wrapper::CommonResponse};

/// 删除物品
/// 根据物品ID删除物品
/// DELETE /item/delete/{itemId}
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractManager(auth): ExtractManager,
    Path(item_id): Path<i64>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match crate::functions::api::item::do_delete(auth, item_id).await {
        Ok(_) => Ok(Json(CommonResponse::new(Ok(EmptyResponse {}))).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
