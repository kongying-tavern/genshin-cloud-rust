use anyhow::Result;

use axum::extract::Json;
use axum::{extract::Path, response::IntoResponse};

use crate::middlewares::ExtractManager;

/// 删除地区
/// DELETE /area/{areaId}
/// 此操作会递归删除，请在前端做二次确认
/// 此操作会把该地区和所属的所有子地区的物品和点位删除
/// 如果点位还属于其他地区的物品，那么这个点位将被保留
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractManager(auth): ExtractManager,
    Path(area_id): Path<i64>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::area::do_delete(auth, area_id).await {
        Ok(resp) => Ok(Json(resp).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
