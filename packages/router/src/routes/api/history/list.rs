use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::history::HistoryListRequest;

/// 历史记录分页查询
/// POST /history/get/list
#[tracing::instrument(skip(auth))]
pub async fn get_list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<HistoryListRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::history::do_get_list(auth, payload).await
    {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
