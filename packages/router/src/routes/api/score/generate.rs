use anyhow::Result;
use serde::{Deserialize, Serialize};

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::score::ScoreGenerateRequest;

/// 生成评分数据
#[tracing::instrument(skip(auth))]
pub async fn generate_score(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(request): Json<ScoreGenerateRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::score::do_generate_score(auth, request).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
