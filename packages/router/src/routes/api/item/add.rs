use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractManager;
use _database::DB_CONN;
use _utils::models::item::ItemAddRequest;

/// 新增物品
/// 新建成功后会返回新物品ID
/// PUT /item/add
#[tracing::instrument(skip(auth))]
pub async fn add(
    ExtractManager(auth): ExtractManager,
    Json(payload): Json<ItemAddRequest>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match crate::functions::api::item::do_add(DB_CONN.wait().as_ref(), auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
