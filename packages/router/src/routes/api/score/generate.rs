use anyhow::Result;
use axum::{extract::Json, http::StatusCode, response::IntoResponse};
use _utils::models::score::ScoreGenerateRequest;
use crate::middlewares::ExtractAuthInfo;

/// 生成评分数据
#[tracing::instrument(skip_all)]
pub async fn generate_score(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(request): Json<ScoreGenerateRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现评分数据生成逻辑
    Ok(())
}
