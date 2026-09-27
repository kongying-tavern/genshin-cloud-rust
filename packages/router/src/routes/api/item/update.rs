use anyhow::Result;

use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
};

use crate::middlewares::ExtractManager;
use _utils::models::item::ItemUpdateData;

/// 修改物品
/// 提供修改同名物品功能，默认关闭
/// POST /item/update/{editSame}
#[tracing::instrument(skip(db, auth))]
pub async fn update(
    State(db): State<crate::routes::SharedDb>,
    ExtractManager(auth): ExtractManager,
    Path(edit_same): Path<i64>,
    Json(payload): Json<Vec<ItemUpdateData>>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::item::do_update(db.as_ref(), auth, edit_same != 0, payload)
        .await
    {
        Ok(resp) => Ok(Json(resp).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
