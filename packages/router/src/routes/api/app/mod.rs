use _utils::models::CommonResponse;
use anyhow::Result;

use crate::middlewares::{ApiError, ExtractAuthInfo};
use axum::{Router, extract::Json, http::StatusCode, response::IntoResponse, routing::post};

/// 触发应用更新（清空 BinaryMD5 缓存，客户端下次轮询重新拉取）
/// POST /app/trigger/update
#[utoipa::path(
    post,
    path = "/api/app/trigger/update",
    tag = "app",
    summary = "触发应用更新",
    responses(
        (status = 200, description = "触发结果", body = inline(CommonResponse<bool>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn trigger_update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::app::do_trigger_update(auth).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

pub async fn router() -> Result<Router> {
    let ret = Router::new().route("/trigger/update", post(trigger_update));
    Ok(ret)
}
