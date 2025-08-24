use anyhow::Result;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 删除公告
#[tracing::instrument(skip_all)]
pub async fn delete_notice(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(notice_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现公告删除逻辑
    Ok(())
}
