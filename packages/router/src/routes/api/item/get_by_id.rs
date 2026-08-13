use anyhow::Result;

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

/// 根据物品ID查询物品
/// 输入ID列表查询，单个查询也用此API
/// POST /item/get/list_by_id
#[tracing::instrument(skip(auth))]
pub async fn get_list_by_id(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<Vec<i64>>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::item::do_get_list_by_id(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
