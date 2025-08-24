use anyhow::Result;
use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::route::{RouteAddRequest, RouteUpdateRequest};

/// 新增路线
/// PUT /route/add
#[tracing::instrument(skip_all)]
pub async fn add(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<RouteAddRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现新增路线的逻辑
    Ok(())
}

/// 修改路线
/// POST /route
#[tracing::instrument(skip_all)]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<RouteUpdateRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现修改路线的逻辑
    Ok(())
}
