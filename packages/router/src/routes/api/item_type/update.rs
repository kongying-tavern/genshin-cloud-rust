use anyhow::Result;

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use axum::{extract::Json, response::IntoResponse};

use _utils::models::{
    common::EmptyResponse, item_type::ItemTypeUpdateData, wrapper::CommonResponse,
};

/// 修改物品类型
/// POST /item_type/update
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<ItemTypeUpdateData>,
) -> Result<impl IntoResponse, ApiError> {
    match crate::functions::api::item_type::do_update(auth, payload).await {
        Ok(_) => Ok(Json(CommonResponse::new(Ok(EmptyResponse {}))).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
