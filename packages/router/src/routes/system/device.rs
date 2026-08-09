use anyhow::Result;
use serde::{Deserialize, Serialize};

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAdmin;
use _utils::models::wrapper::Pagination;
use _utils::types::DeviceSort;

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
    pub user_id: Option<i64>,
}

/// 获取用户设备
/// POST /device/list
#[tracing::instrument(skip(auth))]
pub async fn list(
    ExtractAdmin(auth): ExtractAdmin,
    Json(payload): Json<DeviceListParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let size = payload
        .pagination
        .as_ref()
        .and_then(|p| p.size)
        .unwrap_or(10) as u64;
    let current = payload
        .pagination
        .as_ref()
        .and_then(|p| p.current)
        .unwrap_or(1);

    match _functions::functions::system::device::do_list(
        auth,
        payload.user_id,
        payload.device_id,
        payload.status.map(|s| s as i32),
        payload.sort,
        size,
        current as u64,
    )
    .await
    {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
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
    ExtractAdmin(auth): ExtractAdmin,
    Json(payload): Json<DeviceUpdateParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let status = payload
        .status
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "status required".to_string()))?;
    match _functions::functions::system::device::do_update(auth, payload.id, status as i32).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}
