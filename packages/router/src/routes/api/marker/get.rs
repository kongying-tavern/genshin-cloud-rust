use anyhow::Result;
use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::{marker::MarkerFilterRequest, wrapper::Pagination};

/// 根据各种条件筛选查询点位ID
/// 支持根据末端地区、末端类型、物品来进行查询，三种查询不能同时生效，同时存在时报错
/// POST /marker/get/id
#[tracing::instrument(skip_all)]
pub async fn get_id(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<MarkerFilterRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现根据条件筛选查询点位ID的逻辑
    // 需要验证三种查询条件不能同时生效
    Ok(())
}

/// 根据各种条件筛选查询点位信息
/// 支持根据末端地区、末端类型、物品来进行查询，三种查询不能同时生效，同时存在时报错
/// POST /marker/get/list_by_info
#[tracing::instrument(skip_all)]
pub async fn get_list_by_info(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<MarkerFilterRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现根据条件筛选查询点位信息的逻辑
    // 需要验证三种查询条件不能同时生效
    Ok(())
}

/// 通过ID列表查询点位信息
/// 通过ID列表来进行查询点位信息
/// POST /marker/get/list_by_id
#[tracing::instrument(skip_all)]
pub async fn get_list_by_id(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<Vec<i64>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现通过ID列表查询点位信息的逻辑
    Ok(())
}

/// 分页查询所有点位信息
/// POST /marker/get/page
#[tracing::instrument(skip_all)]
pub async fn get_page(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<Pagination>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现分页查询所有点位信息的逻辑
    Ok(())
}
