use anyhow::Result;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 删除分类
/// 这个操作会递归删除，请在前端做二次确认
/// DELETE /icon_type/delete/{typeId}
#[tracing::instrument(skip(_auth))]
pub async fn delete(
    ExtractAuthInfo(_auth): ExtractAuthInfo,
    Path(type_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
