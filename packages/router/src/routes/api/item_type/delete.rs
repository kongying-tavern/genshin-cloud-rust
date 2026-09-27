use anyhow::Result;

use axum::{
    extract::Path,
    extract::{Json, State},
    response::IntoResponse,
};

use crate::middlewares::ExtractManager;

/// 删除物品类型
/// 批量递归删除物品类型，需在前端做二次确认
/// DELETE /item_type/delete/{itemTypeId}
#[tracing::instrument(skip(db, auth))]
pub async fn delete(
    State(db): State<crate::routes::SharedDb>,
    ExtractManager(auth): ExtractManager,
    Path(item_type_id): Path<i64>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::item_type::do_delete(db.as_ref(), auth, item_type_id).await {
        Ok(resp) => Ok(Json(resp).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
