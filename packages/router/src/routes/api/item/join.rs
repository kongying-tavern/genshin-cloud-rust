use anyhow::Result;

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractManager;

/// 将物品加入某一类型
/// 根据物品ID列表批量加入
/// POST /item/join/{typeId}
#[tracing::instrument(skip(db, auth))]
pub async fn join_type(
    State(db): State<crate::routes::SharedDb>,
    ExtractManager(auth): ExtractManager,
    Path(type_id): Path<i64>,
    Json(payload): Json<Vec<i64>>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::item::do_join_type(db.as_ref(), auth, type_id, payload).await
    {
        Ok(v) => Ok((StatusCode::OK, Json(serde_json::json!(v)))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
