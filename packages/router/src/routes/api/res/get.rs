use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

/// 获取资源信息
/// GET /res/get
///
/// 死代码：Java 侧无此端点、前端无调用方（见 `api::res::do_get` 注释）。
/// 保留路由仅为维持现有路由面，不做任何事。
#[tracing::instrument]
pub async fn get() -> Result<impl IntoResponse, (StatusCode, String)> {
    match crate::functions::api::res::do_get().await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
