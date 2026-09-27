use anyhow::Result;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractManager;

/// 新增地区公用物品
/// 通过ID列表批量添加地区公用物品
/// PUT /item_common/add
#[tracing::instrument(skip(db, auth))]
pub async fn add(
    State(db): State<crate::routes::SharedDb>,
    ExtractManager(auth): ExtractManager,
    Json(payload): Json<Vec<i64>>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::item_common::do_add(db.as_ref(), auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(serde_json::json!(v)))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
