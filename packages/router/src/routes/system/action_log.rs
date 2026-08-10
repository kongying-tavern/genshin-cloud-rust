use anyhow::Result;
use serde::{Deserialize, Serialize};

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAdmin;
use _utils::{models::Pagination, types::ActionLogAction};

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
#[tracing::instrument(skip(auth))]
pub async fn list(
    ExtractAdmin(auth): ExtractAdmin,
    Json(query): Json<ActionLogParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let size_raw = query.pagination.as_ref().and_then(|p| p.size).unwrap_or(10);
    let size: u64 = (if size_raw > 200 { 200 } else { size_raw }) as u64;
    let current = query
        .pagination
        .as_ref()
        .and_then(|p| p.current)
        .unwrap_or(1);

    // 排序参数以 wire 字符串（"createTime+" 等）下发，由业务层映射到列
    let sort = query.sort.map(|sorts| {
        sorts
            .iter()
            .map(|s| serde_json::to_value(s).unwrap_or_default())
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect()
    });

    match _functions::functions::system::action_log::do_list(
        auth,
        query.user_id,
        query.action.map(|a| a as i64),
        query.device_id,
        query.ipv4,
        query.is_error,
        sort,
        size,
        current as u64,
    )
    .await
    {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
