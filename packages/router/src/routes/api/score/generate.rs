use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAdmin;
use _utils::models::score::ScoreGenerateRequest;

/// 生成评分数据（管理员专用：全表扫描 + 写 score_stat，任意登录用户可触发
/// 即 DoS 面，权限在路由层收口；functions 层保留 require_non_anonymous 兜底）。
#[tracing::instrument(skip(auth))]
pub async fn generate_score(
    ExtractAdmin(auth): ExtractAdmin,
    Json(request): Json<ScoreGenerateRequest>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::score::do_generate_score(auth, request).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
