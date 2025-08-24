use anyhow::Result;
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::history::HistoryListRequest;

/// 历史记录分页查询
/// POST /history/get/list
#[tracing::instrument(skip_all)]
pub async fn get_list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<HistoryListRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现历史记录分页查询的逻辑
    Ok(())
}
