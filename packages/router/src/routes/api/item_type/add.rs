use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractManager;
use _utils::models::item_type::ItemTypeAddRequest;

/// 添加物品类型
/// 成功后返回新的类型ID
/// PUT /item_type/add
#[tracing::instrument(skip(auth))]
pub async fn add(
    ExtractManager(auth): ExtractManager,
    Json(payload): Json<ItemTypeAddRequest>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::item_type::do_add(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
