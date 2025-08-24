use anyhow::Result;
use axum::{extract::Json, http::StatusCode, response::IntoResponse};
use _utils::models::score::ScoreDataRequest;
use crate::middlewares::ExtractAuthInfo;

/// 获取评分数据
#[tracing::instrument(skip_all)]
pub async fn get_score_data(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(request): Json<ScoreDataRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现评分数据获取逻辑
    Ok(())
}
