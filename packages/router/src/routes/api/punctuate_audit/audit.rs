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
    match _functions::functions::api::punctuate_audit::do_pass(
        auth,
        serde_json::json!({"punctuate_id": punctuate_id}),
    )
    .await
    {
        Ok(_) => Ok((StatusCode::OK, Json(serde_json::json!({})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}

/// 驳回点位审核
/// POST /punctuate_audit/reject/{punctuateId}
#[tracing::instrument(skip(auth))]
pub async fn reject(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(punctuate_id): Path<i64>,
    Json(audit_remark): Json<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::punctuate_audit::do_reject(
        auth,
        serde_json::json!({"punctuate_id": punctuate_id, "remark": audit_remark}),
    )
    .await
    {
        Ok(_) => Ok((StatusCode::OK, Json(serde_json::json!({})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
