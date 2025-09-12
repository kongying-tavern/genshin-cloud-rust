use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;

/// 将暂存点位提交审核
/// PUT /punctuate/push/{authorId}
#[tracing::instrument(skip(auth))]
pub async fn push(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(author_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::punctuate::do_push(
        auth,
        serde_json::json!({"author_id": author_id}),
    )
    .await
    {
        Ok(v) => Ok((StatusCode::OK, Json(serde_json::json!(v)))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}

/// 删除自己未通过的提交点位
/// DELETE /punctuate/delete/{authorId}/{punctuateId}
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path((author_id, punctuate_id)): Path<(i64, i64)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::punctuate::do_delete(
        auth,
        serde_json::json!({"author_id": author_id, "punctuate_id": punctuate_id}),
    )
    .await
    {
        Ok(v) => Ok((StatusCode::OK, Json(serde_json::json!(v)))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
