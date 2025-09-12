use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::punctuate::PunctuateData;

/// 修改自身未提交的暂存点位
/// POST /punctuate/
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<PunctuateData>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::punctuate::do_update(
        auth,
        payload,
    )
    .await
    {
        Ok(v) => Ok((StatusCode::OK, Json(serde_json::json!(v)))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}

/// 提交暂存点位
/// PUT /punctuate/
#[tracing::instrument(skip(auth))]
pub async fn submit(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<PunctuateData>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::punctuate::do_submit(
        auth,
        payload,
    )
    .await
    {
        Ok(v) => Ok((StatusCode::OK, Json(serde_json::json!(v)))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
