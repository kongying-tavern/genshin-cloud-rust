use anyhow::Result;
use serde::{Deserialize, Serialize};

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::score::ScoreDataRequest;

/// 获取评分数据
#[tracing::instrument(skip(auth))]
pub async fn get_score_data(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(request): Json<ScoreDataRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::score::do_get_score_data(auth, request).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
