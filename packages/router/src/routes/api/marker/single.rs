use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::marker::{MarkerAddRequest, MarkerUpdateData};
use _utils::models::{CommonResponse, MarkerEmptyResponse};

/// 新增点位
/// PUT /marker/single
#[utoipa::path(
    put,
    path = "/api/marker/single",
    tag = "marker",
    summary = "新增点位",
    request_body = MarkerAddRequest,
    responses(
        (status = 200, description = "新点位 ID", body = inline(CommonResponse<i64>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn add_single(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<MarkerAddRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::marker::do_add_single(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(serde_json::json!(v)))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 修改点位
/// POST /marker/single
#[utoipa::path(
    post,
    path = "/api/marker/single",
    tag = "marker",
    summary = "修改点位",
    request_body = MarkerUpdateData,
    responses(
        (status = 200, description = "更新结果", body = inline(CommonResponse<MarkerEmptyResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn update_single(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<MarkerUpdateData>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::marker::do_update_single(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(serde_json::json!(v)))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
