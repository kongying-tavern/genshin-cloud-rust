use _utils::models::CommonResponse;
use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::marker_link::{MarkerLinkGraphRequest, MarkerLinkListRequest};

/// 点位关联列表
/// POST /marker_link/get/list
#[utoipa::path(
    post,
    path = "/api/marker_link/get/list",
    tag = "marker-link",
    summary = "点位关联列表",
    request_body = MarkerLinkListRequest,
    responses(
        (status = 200, description = "关联列表", body = inline(CommonResponse<serde_json::Value>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn get_list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<MarkerLinkListRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // removed local alias
    match _functions::functions::api::marker_link::do_get_list(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 点位关联图数据
/// POST /marker_link/get/graph
#[utoipa::path(
    post,
    path = "/api/marker_link/get/graph",
    tag = "marker-link",
    summary = "点位关联图数据",
    request_body = MarkerLinkGraphRequest,
    responses(
        (status = 200, description = "图数据", body = inline(CommonResponse<serde_json::Value>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn get_graph(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<MarkerLinkGraphRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::marker_link::do_get_graph(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
