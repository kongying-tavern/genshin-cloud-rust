use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::punctuate::PunctuateData;
use _utils::types::MarkerPunctuateStatus;

/// 将暂存点位提交审核（COMMIT：Pending/Rejected → Reviewing）
/// PUT /punctuate/push/{punctuateId}
///
/// 接收一个最小的 PunctuateData body，status 固定为 Reviewing。
#[tracing::instrument(skip(auth))]
pub async fn push(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(punctuate_id): Path<i64>,
    Json(mut payload): Json<PunctuateData>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    payload.punctuate_id = punctuate_id as f64;
    payload.status = MarkerPunctuateStatus::Reviewing;
    match _functions::functions::api::punctuate::do_submit(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}

/// 删除自己未通过的提交点位
/// DELETE /punctuate/delete/{authorId}/{punctuateId}
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path((_author_id, punctuate_id)): Path<(i64, i64)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::punctuate::do_delete(auth, punctuate_id).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}
