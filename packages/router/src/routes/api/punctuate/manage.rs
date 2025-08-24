use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::punctuate::PunctuateData;

/// 修改自身未提交的暂存点位
/// POST /punctuate/
#[tracing::instrument(skip_all)]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<PunctuateData>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现修改自身未提交的暂存点位的逻辑
    Ok(())
}

/// 提交暂存点位
/// PUT /punctuate/
#[tracing::instrument(skip_all)]
pub async fn submit(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<PunctuateData>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现提交暂存点位的逻辑
    Ok(())
}
