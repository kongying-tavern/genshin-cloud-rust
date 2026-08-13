use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::{CommonResponse, MarkerListResponse, MarkerVO};
use _utils::models::{marker::MarkerFilterRequest, wrapper::Pagination};

/// 根据各种条件筛选查询点位ID
/// 支持根据末端地区、末端类型、物品来进行查询，三种查询不能同时生效，同时存在时报错
/// POST /marker/get/id
#[utoipa::path(
    post,
    path = "/api/marker/get/id",
    tag = "marker",
    summary = "按条件筛选点位 ID",
    request_body = MarkerFilterRequest,
    responses(
        (status = 200, description = "点位 ID 列表", body = inline(CommonResponse<Vec<i64>>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn get_id(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<MarkerFilterRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::marker::do_get_id(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 根据各种条件筛选查询点位信息
/// 支持根据末端地区、末端类型、物品来进行查询，三种查询不能同时生效，同时存在时报错
/// POST /marker/get/list_by_info
#[utoipa::path(
    post,
    path = "/api/marker/get/list_byinfo",
    tag = "marker",
    summary = "按条件筛选点位信息",
    request_body = MarkerFilterRequest,
    responses(
        (status = 200, description = "点位列表", body = inline(CommonResponse<Vec<MarkerVO>>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn get_list_by_info(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<MarkerFilterRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::marker::do_get_list_by_info(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 通过ID列表查询点位信息
/// 通过ID列表来进行查询点位信息
/// POST /marker/get/list_by_id
#[utoipa::path(
    post,
    path = "/api/marker/get/list_byid",
    tag = "marker",
    summary = "通过 ID 列表查询点位信息",
    request_body = Vec<i64>,
    responses(
        (status = 200, description = "点位列表", body = inline(CommonResponse<Vec<MarkerVO>>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn get_list_by_id(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<Vec<i64>>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::marker::do_get_list_by_id(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 分页查询所有点位信息
/// POST /marker/get/page
#[utoipa::path(
    post,
    path = "/api/marker/get/page",
    tag = "marker",
    summary = "分页查询所有点位",
    request_body = Pagination,
    responses(
        (status = 200, description = "点位分页列表", body = inline(CommonResponse<MarkerListResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn get_page(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<Pagination>,
) -> Result<impl IntoResponse, ApiError> {
    // use axum::Json as AxumJson; (removed duplicate alias)
    match _functions::functions::api::marker::do_get_page(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
