use anyhow::Result;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 将暂存点位提交审核
/// PUT /punctuate/push/{authorId}
#[tracing::instrument(skip(auth))]
pub async fn push(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(author_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现将暂存点位提交审核的逻辑
    Ok(())
}

/// 删除自己未通过的提交点位
/// DELETE /punctuate/delete/{authorId}/{punctuateId}
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path((author_id, punctuate_id)): Path<(i64, i64)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现删除自己未通过的提交点位的逻辑
    Ok(())
}
