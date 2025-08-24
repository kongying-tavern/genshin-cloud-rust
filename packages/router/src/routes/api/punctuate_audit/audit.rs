use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;

/// 通过点位审核
/// POST /punctuate_audit/pass/{punctuateId}
#[tracing::instrument(skip(auth))]
pub async fn pass(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(punctuate_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现通过点位审核的逻辑
    Ok(())
}

/// 驳回点位审核
/// POST /punctuate_audit/reject/{punctuateId}
#[tracing::instrument(skip(auth))]
pub async fn reject(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(punctuate_id): Path<i64>,
    Json(audit_remark): Json<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现驳回点位审核的逻辑
    Ok(())
}
