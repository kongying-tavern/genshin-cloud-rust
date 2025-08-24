use anyhow::Result;
use serde::{Deserialize, Serialize};

use axum::{
    extract::{Json, Query},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;
use _utils::{
    models::Pagination,
    types::{ActionLogAction, SystemUserRole},
};

/// 格式：字段+ 字段-
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionLogSort {
    #[serde(rename = "createTime+")]
    CreateTime,
    #[serde(rename = "createTime-")]
    CreateTimeReverse,
    #[serde(rename = "deviceId+")]
    DeviceId,
    #[serde(rename = "deviceId-")]
    DeviceIdReverse,
    #[serde(rename = "id+")]
    Id,
    #[serde(rename = "id-")]
    IdReverse,
    #[serde(rename = "ipv4+")]
    Ipv4,
    #[serde(rename = "ipv4-")]
    Ipv4Reverse,
    #[serde(rename = "isError+")]
    IsError,
    #[serde(rename = "isError-")]
    IsErrorReverse,
    #[serde(rename = "updateTime+")]
    UpdateTime,
    #[serde(rename = "updateTime-")]
    UpdateTimeReverse,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionLogParams {
    /// 设备状态
    pub action: Option<ActionLogAction>,
    #[serde(flatten)]
    pub pagination: Option<Pagination>,
    /// 设备ID
    pub device_id: Option<String>,
    /// IPv4
    pub ipv4: Option<String>,
    /// 是否是错误日志
    pub is_error: Option<bool>,
    /// 排序
    pub sort: Option<Vec<ActionLogSort>>,
    /// 用户ID
    pub user_id: Option<i64>,
}

/// 获取操作日志
/// POST /action_log/list
#[tracing::instrument(skip_all)]
pub async fn list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Query(query): Query<ActionLogParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if auth.info.role_id != SystemUserRole::Admin {
        return Ok((axum::http::StatusCode::FORBIDDEN, "Forbidden".to_string()).into_response());
    }

    Ok(Json(()).into_response())
}
