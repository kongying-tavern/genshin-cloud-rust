use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use _utils::models::history::HistoryListRequest;
use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};

/// 历史记录分页查询
/// POST /history/get/list
#[tracing::instrument(skip(auth))]
pub async fn get_list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<HistoryListRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::history::do_get_list(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
