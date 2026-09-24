use anyhow::Result;

use axum::{extract::Json, response::IntoResponse};

use crate::middlewares::ExtractManager;
use _utils::models::item_type::ItemTypeUpdateData;

/// 修改物品类型
/// POST /item_type/update
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractManager(auth): ExtractManager,
    Json(payload): Json<ItemTypeUpdateData>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match crate::functions::api::item_type::do_update(auth, payload).await {
        Ok(resp) => Ok(Json(resp).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
