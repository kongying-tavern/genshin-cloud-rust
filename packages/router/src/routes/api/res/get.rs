use anyhow::Result;

use crate::middlewares::ApiError;
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use _utils::models::CommonResponse;

/// 获取资源信息
/// GET /res/get
///
/// 死代码：Java 侧无此端点、前端无调用方（见 `api::res::do_get` 注释）。
/// 保留路由仅为维持现有路由面，不做任何事。
#[utoipa::path(
    get,
    path = "/api/res/get",
    tag = "res",
    summary = "获取资源信息",
    responses(
        (status = 200, description = "资源信息", body = inline(CommonResponse<utoipa::TupleUnit>)),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument]
pub async fn get() -> Result<impl IntoResponse, ApiError> {
    match crate::functions::api::res::do_get().await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
