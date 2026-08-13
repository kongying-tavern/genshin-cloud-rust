use _utils::models::CommonResponse;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAdmin, api_error};
use _utils::models::wrapper::Pagination;
use _utils::types::DeviceSort;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
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
#[utoipa::path(
    post,
    path = "/system/device/list",
    tag = "system",
    summary = "获取用户设备",
    request_body = DeviceListParams,
    responses(
        (status = 200, description = "设备分页列表", body = inline(CommonResponse<serde_json::Value>)),
        (status = 401, description = "未登录或无权访问"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn list(
    ExtractAdmin(auth): ExtractAdmin,
    AppJson(payload): AppJson<DeviceListParams>,
) -> Result<impl IntoResponse, ApiError> {
    let size_raw = payload
        .pagination
        .as_ref()
        .and_then(|p| p.size)
        .unwrap_or(10);
    let size: u64 = (if size_raw > 200 { 200 } else { size_raw }) as u64;
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
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceUpdateParams {
    /// ID
    pub id: i64,
    pub status: Option<i64>,
}

/// 更新用户设备信息
/// POST /device/update
#[utoipa::path(
    post,
    path = "/system/device/update",
    tag = "system",
    summary = "更新用户设备信息",
    request_body = DeviceUpdateParams,
    responses(
        (status = 200, description = "更新结果", body = inline(CommonResponse<utoipa::TupleUnit>)),
        (status = 401, description = "未登录或无权访问"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractAdmin(auth): ExtractAdmin,
    AppJson(payload): AppJson<DeviceUpdateParams>,
) -> Result<impl IntoResponse, ApiError> {
    let status = payload
        .status
        .ok_or_else(|| api_error(StatusCode::BAD_REQUEST, "status required"))?;
    match _functions::functions::system::device::do_update(auth, payload.id, status as i32).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
