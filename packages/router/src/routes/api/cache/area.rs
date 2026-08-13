use anyhow::Result;

use crate::middlewares::{ApiError, ExtractAuthInfo};
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use _utils::models::{CommonResponse, common::EmptyResponse};

/// 删除地区缓存
#[utoipa::path(
    delete,
    path = "/api/cache/area",
    tag = "cache",
    summary = "删除地区缓存",
    responses(
        (status = 200, description = "删除结果", body = inline(CommonResponse<EmptyResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn delete_area_cache(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::cache::do_delete_area_cache(auth).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
