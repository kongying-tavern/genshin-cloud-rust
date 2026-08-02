use anyhow::Result;
use serde::{Deserialize, Serialize};

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::{ExtractAdmin, ExtractAuthInfo};
use _utils::{models::wrapper::Pagination, types::AccessPolicyItemEnum};

/// 获取用户邀请列表的请求参数
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitationListRequest {
    /// 邀请码
    pub code: Option<String>,
    #[serde(flatten)]
    pub pagination: Option<Pagination>,
    /// 排序
    pub sort: Option<Vec<InvitationSort>>,
    /// 用户名
    pub username: Option<String>,
}

/// 邀请排序枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InvitationSort {
    #[serde(rename = "createTime+")]
    CreateTime,
    #[serde(rename = "createTime-")]
    CreateTimeReverse,
    #[serde(rename = "id+")]
    Id,
    #[serde(rename = "id-")]
    IdReverse,
    #[serde(rename = "updateTime+")]
    UpdateTime,
    #[serde(rename = "updateTime-")]
    UpdateTimeReverse,
    #[serde(rename = "username+")]
    Username,
    #[serde(rename = "username-")]
    UsernameReverse,
}

/// 新增/更新用户邀请的请求参数
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitationUpdateRequest {
    /// 权限策略
    pub access_policy: Vec<AccessPolicyItemEnum>,
    /// 邀请码
    pub code: String,
    /// 备注
    pub remark: String,
    /// 角色列表
    pub role_id: i64,
    /// 用户名
    pub username: String,
}

/// 使用用户邀请的请求参数
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitationConsumeRequest {
    /// 邀请码
    pub code: String,
    /// 用户名
    pub username: String,
}

/// 检查用户邀请数据的请求参数
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitationInfoRequest {
    /// 邀请码
    pub code: String,
}

/// 获取用户邀请列表
/// POST /invitation/list
#[tracing::instrument(skip(auth))]
pub async fn list(
    ExtractAdmin(auth): ExtractAdmin,
    Json(payload): Json<InvitationListRequest>,
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

    match _functions::functions::system::invitation::do_list(
        auth,
        payload.code,
        payload.username,
        size,
        current as u64,
    )
    .await
    {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}

/// 新增/更新用户邀请
/// POST /invitation/update
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractAdmin(auth): ExtractAdmin,
    Json(payload): Json<InvitationUpdateRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::system::invitation::do_update(
        auth,
        payload.code,
        Some(payload.role_id),
        Some(payload.remark),
    )
    .await
    {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}

/// 检查用户邀请数据
/// POST /invitation/info
#[tracing::instrument(skip(auth))]
pub async fn info(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<InvitationInfoRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::system::invitation::do_info(auth, payload.code).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}

/// 使用用户邀请
/// POST /invitation/consume
#[tracing::instrument(skip(auth))]
pub async fn consume(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<InvitationConsumeRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::system::invitation::do_consume(auth, payload.code).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}

/// 删除用户邀请
/// DELETE /invitation/{invitation_id}
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAdmin(auth): ExtractAdmin,
    Path(invitation_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::system::invitation::do_delete(auth, invitation_id).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}
