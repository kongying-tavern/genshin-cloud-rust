use anyhow::Result;
use serde::{Deserialize, Serialize};

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::wrapper::Pagination;

/// 打点审核筛选请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PunctuateAuditFilterRequest {
    /// 地区 ID 列表
    pub area_id_list: Option<Vec<i64>>,
    /// 提交者 ID 列表
    pub author_list: Option<Vec<i64>>,
    /// 物品 ID 列表
    pub item_id_list: Option<Vec<i64>>,
    /// 类型 ID 列表
    pub type_id_list: Option<Vec<i64>>,
}

/// 根据各种条件筛选打点ID
/// POST /punctuate_audit/get/id
#[tracing::instrument(skip_all)]
pub async fn get_id(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<PunctuateAuditFilterRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现根据条件筛选打点ID的逻辑
    Ok(())
}

/// 根据各种条件筛选打点信息
/// POST /punctuate_audit/get/list_byinfo
#[tracing::instrument(skip_all)]
pub async fn get_list_by_info(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<PunctuateAuditFilterRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现根据条件筛选打点信息的逻辑
    Ok(())
}

/// 通过打点ID列表查询打点信息
/// POST /punctuate_audit/get/list_byid
#[tracing::instrument(skip_all)]
pub async fn get_list_by_id(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<Vec<i64>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现通过ID列表查询打点信息的逻辑
    Ok(())
}

/// 分页查询所有打点信息（包括暂存）
/// POST /punctuate_audit/get/page/all
#[tracing::instrument(skip_all)]
pub async fn get_page_all(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<Pagination>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现分页查询所有打点信息的逻辑
    Ok(())
}
