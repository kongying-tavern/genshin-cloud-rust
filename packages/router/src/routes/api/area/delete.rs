use anyhow::Result;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 删除地区
/// DELETE /area/{areaId}
/// 此操作会递归删除，请在前端做二次确认
/// 此操作会把该地区和所属的所有子地区的物品和点位删除
/// 如果点位还属于其他地区的物品，那么这个点位将被保留
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(area_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
