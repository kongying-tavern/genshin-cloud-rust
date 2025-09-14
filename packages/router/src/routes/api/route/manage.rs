use anyhow::Result;
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::route::{RouteAddRequest, RouteUpdateRequest};

/// 新增路线
/// PUT /route/add
#[tracing::instrument(skip(auth))]
pub async fn add(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<RouteAddRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::route::do_add(auth, payload).await {
        Ok(id) => Ok((StatusCode::OK, Json(serde_json::json!({"id": id})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}

/// 修改路线
/// POST /route
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<RouteUpdateRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match crate::functions::api::route::do_update(auth, payload).await {
        Ok(_) => Ok((StatusCode::OK, Json(serde_json::json!({})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
