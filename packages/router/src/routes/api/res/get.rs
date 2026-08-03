use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

/// 获取资源信息
/// GET /res/get
#[tracing::instrument]
pub async fn get() -> Result<impl IntoResponse, (StatusCode, String)> {
    match crate::functions::api::res::do_get().await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
