use anyhow::Result;
use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;

/// 删除提交点位
/// DELETE /punctuate_audit/delete/{punctuateId}
#[tracing::instrument(skip_all)]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(punctuate_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现删除提交点位的逻辑
    Ok(())
}
