use anyhow::Result;
use serde::{Deserialize, Serialize};

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::{models::wrapper::Pagination, types::SystemUserRole};

/// 格式：字段+ 字段-
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeviceSort {
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
    #[serde(rename = "lastLoginTime+")]
    LastLoginTime,
    #[serde(rename = "lastLoginTime-")]
    LastLoginTimeReverse,
    #[serde(rename = "status+")]
    Status,
    #[serde(rename = "status-")]
    StatusReverse,
    #[serde(rename = "updateTime+")]
    UpdateTime,
    #[serde(rename = "updateTime-")]
    UpdateTimeReverse,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceListParams {
    #[serde(flatten)]
    pub pagination: Option<Pagination>,
    /// 设备ID
    pub device_id: Option<String>,
    /// IPv4
    pub ipv4: Option<String>,
    /// 排序
    pub sort: Option<Vec<DeviceSort>>,
    /// 设备状态
    pub status: Option<i64>,
    /// 用户ID
    pub user_id: i64,
}

/// 获取用户设备
/// POST /device/list
#[tracing::instrument(skip(auth))]
pub async fn list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<DeviceListParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if auth.info.role_id != SystemUserRole::Admin {
        return Ok((axum::http::StatusCode::FORBIDDEN, "Forbidden".to_string()).into_response());
    }

    Ok(Json(()).into_response())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceUpdateParams {
    /// ID
    pub id: i64,
    pub status: Option<i64>,
}

/// 更新用户设备信息
/// POST /device/update
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<DeviceUpdateParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if auth.info.role_id != SystemUserRole::Admin {
        return Ok((axum::http::StatusCode::FORBIDDEN, "Forbidden".to_string()).into_response());
    }

    Ok(Json(()).into_response())
}
