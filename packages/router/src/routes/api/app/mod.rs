use anyhow::Result;

use axum::{Router, extract::Json, http::StatusCode, response::IntoResponse, routing::post};

use crate::middlewares::ExtractAuthInfo;

/// 触发应用更新（清空 BinaryMD5 缓存，客户端下次轮询重新拉取）
/// POST /app/trigger/update
#[tracing::instrument(skip(auth))]
pub async fn trigger_update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::app::do_trigger_update(auth).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}

pub async fn router() -> Result<Router> {
    let ret = Router::new().route("/trigger/update", post(trigger_update));
    Ok(ret)
}
