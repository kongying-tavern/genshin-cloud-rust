use anyhow::Result;
use serde::{Deserialize, Serialize};

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;
use _utils::{models::wrapper::Pagination, types::AccessPolicyItemEnum};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UserSort {
    #[serde(rename = "createTime+")]
    CreateTime,
    #[serde(rename = "id+")]
    Id,
    #[serde(rename = "nickname+")]
    Nickname,
    #[serde(rename = "createTime-")]
    SortCreateTime,
    #[serde(rename = "id-")]
    SortId,
    #[serde(rename = "nickname-")]
    SortNickname,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRegisterParams {
    /// 权限策略
    pub access_policy: Vec<AccessPolicyItemEnum>,
    /// 头像链接
    pub logo: String,
    /// 备注
    pub remark: String,
    /// 角色列表
    pub role_id: i64,
    /// 用户名
    pub username: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRegisterQqParams {
    /// 权限策略
    pub access_policy: Vec<AccessPolicyItemEnum>,
    /// 头像链接
    pub logo: String,
    /// 备注
    pub remark: String,
    /// 角色列表
    pub role_id: i64,
    /// 用户名
    pub username: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserUpdateParams {
    /// 权限策略
    pub access_policy: Vec<AccessPolicyItemEnum>,
    /// 头像链接
    pub logo: String,
    /// 昵称
    pub nickname: String,
    /// 手机号
    pub phone: String,
    /// QQ
    pub qq: String,
    /// 备注
    pub remark: String,
    /// 角色列表
    pub role_id: i64,
    /// 用户ID
    pub user_id: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserUpdatePasswordParams {
    /// 权限策略
    pub access_policy: Vec<AccessPolicyItemEnum>,
    /// ID
    pub id: i64,
    /// 头像链接
    pub logo: String,
    pub old_password: String,
    /// 备注
    pub remark: String,
    /// 角色列表
    pub role_id: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserUpdatePasswordByAdminParams {
    /// 新密码
    pub password: String,
    /// 用户ID
    pub user_id: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserListParams {
    #[serde(flatten)]
    pub pagination: Pagination,
    /// 昵称
    pub nickname: String,
    /// 角色ID
    pub role_ids: Option<Vec<i64>>,
    pub sort: Option<Vec<UserSort>>,
    /// 用户名
    pub username: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserKickOutParams {
    pub work_id: String,
}

/// 注册用户
/// POST /user/register
#[tracing::instrument(skip_all)]
pub async fn register(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<UserRegisterParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}

/// 用QQ注册用户
/// POST /user/register/qq
#[tracing::instrument(skip_all)]
pub async fn register_qq(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<UserRegisterQqParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}

/// 获取用户信息
/// GET /user/info/{userId}
#[tracing::instrument(skip_all)]
pub async fn get_info(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(user_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}

/// 更新用户信息
/// POST /user/update
#[tracing::instrument(skip_all)]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<UserUpdateParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}

/// 更新用户密码
/// POST /user/update_password
#[tracing::instrument(skip_all)]
pub async fn update_password(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<UserUpdatePasswordParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}

/// 更新用户密码（管理员）
/// POST /user/update_password_by_admin
#[tracing::instrument(skip_all)]
pub async fn update_password_by_admin(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<UserUpdatePasswordByAdminParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}

/// 删除用户
/// DELETE /user/{workId}
#[tracing::instrument(skip_all)]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(work_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}

/// 用户信息(批量查询)
/// POST /user/info/list
#[tracing::instrument(skip_all)]
pub async fn list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<UserListParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}

/// 踢出用户
/// DELETE /user/kick_out/{workId}
#[tracing::instrument(skip_all)]
pub async fn kick_out(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(work_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
