use anyhow::Result;

use axum::{
    extract::Path,
    extract::{Json, State},
    response::IntoResponse,
};

use crate::middlewares::ExtractManager;

/// 删除物品
/// 根据物品ID删除物品
/// DELETE /item/delete/{itemId}
#[tracing::instrument(skip(db, auth))]
pub async fn delete(
    State(db): State<crate::routes::SharedDb>,
    ExtractManager(auth): ExtractManager,
    Path(item_id): Path<i64>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match crate::functions::api::item::do_delete(db.as_ref(), auth, item_id).await {
        Ok(resp) => Ok(Json(resp).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
