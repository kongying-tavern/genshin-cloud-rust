use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractManager;
use _utils::models::{
    common::EmptyResponse, item_type::ItemTypeUpdateData, wrapper::CommonResponse,
};

/// 修改物品类型
/// POST /item_type/update
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractManager(auth): ExtractManager,
    Json(payload): Json<ItemTypeUpdateData>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match crate::functions::api::item_type::do_update(auth, payload).await {
        Ok(_) => Ok(Json(CommonResponse::new(Ok(EmptyResponse {}))).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
