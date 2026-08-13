use anyhow::Result;

use axum::extract::Json;
use axum::{extract::Path, response::IntoResponse};

use crate::middlewares::{ApiError, ExtractAuthInfo};
use _utils::models::{common::EmptyResponse, wrapper::CommonResponse};

/// 删除地区
/// DELETE /area/{areaId}
/// 此操作会递归删除，请在前端做二次确认
/// 此操作会把该地区和所属的所有子地区的物品和点位删除
/// 如果点位还属于其他地区的物品，那么这个点位将被保留
#[utoipa::path(
    delete,
    path = "/api/area/{area_id}",
    tag = "area",
    summary = "删除地区（递归删除子地区）",
    params(("area_id" = i64, Path, description = "地区 ID")),
    responses(
        (status = 200, description = "删除结果", body = inline(CommonResponse<EmptyResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(area_id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::area::do_delete(auth, area_id).await {
        Ok(_) => Ok(Json(CommonResponse::new(Ok(EmptyResponse {}))).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
