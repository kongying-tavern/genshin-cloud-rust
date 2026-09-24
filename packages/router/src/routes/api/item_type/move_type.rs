use anyhow::Result;

use axum::{
    extract::{Json, Path},
    response::IntoResponse,
};

use crate::middlewares::ExtractManager;

/// 批量移动类型为目标类型的子类型
/// 将类型批量移动到某个类型下作为其子类型
/// POST /item_type/move/{targetTypeId}
#[tracing::instrument(skip(auth))]
pub async fn move_to_target(
    ExtractManager(auth): ExtractManager,
    Path(target_type_id): Path<i64>,
    Json(payload): Json<Vec<i64>>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::item_type::do_move_to_target(auth, target_type_id, payload)
        .await
    {
        Ok(resp) => Ok(Json(resp).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
