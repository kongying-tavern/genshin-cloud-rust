use _utils::models::CommonResponse;
use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAdmin};
use _utils::models::score::ScoreGenerateRequest;

/// 生成评分数据（管理员专用：全表扫描 + 写 score_stat，任意登录用户可触发
/// 即 DoS 面，权限在路由层收口；functions 层保留 require_non_anonymous 兜底）。
#[utoipa::path(
    post,
    path = "/api/score/generate",
    tag = "score",
    summary = "生成评分数据（管理员专用）",
    request_body = ScoreGenerateRequest,
    responses(
        (status = 200, description = "生成结果", body = inline(CommonResponse<String>)),
        (status = 401, description = "未登录或无权访问"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn generate_score(
    ExtractAdmin(auth): ExtractAdmin,
    AppJson(request): AppJson<ScoreGenerateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::score::do_generate_score(auth, request).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
