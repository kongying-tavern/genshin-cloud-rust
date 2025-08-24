use anyhow::Result;
use serde::{Deserialize, Serialize};

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::wrapper::Pagination;

/// 路线筛选查询请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteSearchRequest {
    /// 创建人 id，创建人 id，此字段不能与昵称模糊搜索字段共存
    pub creator_id: Option<String>,
    /// 创建人昵称模糊搜索字段，创建人昵称模糊搜索字段，此字段不能与创建人 id 字段共存
    pub creator_nickname_part: Option<String>,
    /// 路线名称模糊搜索字段
    pub name_part: Option<String>,
    /// 分页参数
    #[serde(flatten)]
    pub page: Pagination,
}

/// 分页查询所有路线信息
/// 分页查询所有路线信息，会根据当前角色决定不同的显隐等级
/// POST /route/get/page
#[tracing::instrument(skip(auth))]
pub async fn get_page(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<Pagination>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现分页查询所有路线信息的逻辑
    Ok(())
}

/// 根据条件筛选分页查询路线信息
/// 根据条件筛选分页查询路线信息，会根据当前角色决定不同的显隐等级
/// POST /route/get/search
#[tracing::instrument(skip(auth))]
pub async fn get_search(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<RouteSearchRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现根据条件筛选分页查询路线信息的逻辑
    Ok(())
}

/// 根据id列表查询路线信息
/// 根据id列表查询路线信息，会根据当前角色决定不同的显隐等级
/// POST /route/get/list_by_id
#[tracing::instrument(skip(auth))]
pub async fn get_list_by_id(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<Vec<f64>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现根据id列表查询路线信息的逻辑
    Ok(())
}
