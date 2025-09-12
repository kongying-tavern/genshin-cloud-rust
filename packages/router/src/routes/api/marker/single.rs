use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::marker::{MarkerAddRequest, MarkerUpdateData};

/// 新增点位
/// PUT /marker/single
#[tracing::instrument(skip(auth))]
pub async fn add_single(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<MarkerAddRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::marker::do_add_single(
        auth,
        payload,
    )
    .await
    {
        Ok(v) => Ok((StatusCode::OK, Json(serde_json::json!(v)))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}

/// 修改点位
/// POST /marker/single
#[tracing::instrument(skip(auth))]
pub async fn update_single(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<MarkerUpdateData>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::marker::do_update_single(
        auth,
        payload,
    )
    .await
    {
        Ok(v) => Ok((StatusCode::OK, Json(serde_json::json!(v)))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
