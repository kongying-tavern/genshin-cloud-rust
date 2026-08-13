use _utils::models::CommonResponse;
use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::score::ScoreDataRequest;

/// 获取评分数据
#[utoipa::path(
    post,
    path = "/api/score/data",
    tag = "score",
    summary = "获取评分数据",
    request_body = ScoreDataRequest,
    responses(
        (status = 200, description = "评分数据", body = inline(CommonResponse<serde_json::Value>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
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
