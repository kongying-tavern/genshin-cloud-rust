use crate::middlewares::{ApiError, ExtractAuthInfo};
use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use _utils::models::{AreaVO, CommonResponse};

/// 获取单个地区信息
/// POST /area/get/{areaId}
#[utoipa::path(
    post,
    path = "/api/area/get/{area_id}",
    tag = "area",
    summary = "获取单个地区信息",
    params(("area_id" = i64, Path, description = "地区 ID")),
    responses(
        (status = 200, description = "地区信息", body = inline(CommonResponse<AreaVO>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn get(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(area_id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::area::do_get(auth, area_id).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
