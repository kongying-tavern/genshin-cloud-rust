use anyhow::Result;
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::{wrapper::Pagination, RouteSearchRequest};

// RouteSearchRequest is defined in `_utils::models::route` and re-exported as
// `_utils::models::RouteSearchRequest`.

/// 分页查询所有路线信息
/// 分页查询所有路线信息，会根据当前角色决定不同的显隐等级
/// POST /route/get/page
#[tracing::instrument(skip(auth))]
pub async fn get_page(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<Pagination>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::route::do_get_page(
        auth,
        payload,
    )
    .await
    {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}

/// 根据条件筛选分页查询路线信息
/// 根据条件筛选分页查询路线信息，会根据当前角色决定不同的显隐等级
/// POST /route/get/search
#[tracing::instrument(skip(auth))]
pub async fn get_search(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<RouteSearchRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::route::do_get_search(
        auth,
        payload,
    )
    .await
    {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}

/// 根据id列表查询路线信息
/// 根据id列表查询路线信息，会根据当前角色决定不同的显隐等级
/// POST /route/get/list_by_id
#[tracing::instrument(skip(auth))]
pub async fn get_list_by_id(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<Vec<f64>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::route::do_get_list_by_id(
        auth,
        payload,
    )
    .await
    {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
