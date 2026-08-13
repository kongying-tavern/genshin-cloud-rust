use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use _utils::models::score::ScoreDataRequest;
use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};

/// 获取评分数据
#[tracing::instrument(skip(auth))]
pub async fn get_score_data(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(request): AppJson<ScoreDataRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::score::do_get_score_data(auth, request).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
