use anyhow::Result;
use serde::{Deserialize, Serialize};

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;
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
#[tracing::instrument(skip_all)]
pub async fn list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<InvitationListRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}

/// 新增/更新用户邀请
/// POST /invitation/update
#[tracing::instrument(skip_all)]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<InvitationUpdateRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}

/// 检查用户邀请数据
/// POST /invitation/info
#[tracing::instrument(skip_all)]
pub async fn info(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<InvitationInfoRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}

/// 使用用户邀请
/// POST /invitation/consume
#[tracing::instrument(skip_all)]
pub async fn consume(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<InvitationConsumeRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}

/// 删除用户邀请
/// DELETE /invitation/{invitation_id}
#[tracing::instrument(skip_all)]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(invitation_id): Path<u64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
