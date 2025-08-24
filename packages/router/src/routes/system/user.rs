use anyhow::Result;
use serde::{Deserialize, Serialize};

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;
use _functions::functions::system::user::*;
use _utils::{
    models::Pagination,
    types::{AccessPolicyItemEnum, SystemUserRole},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UserSort {
    #[serde(rename = "createTime+")]
    CreateTime,
    #[serde(rename = "createTime-")]
    CreateTimeReverse,
    #[serde(rename = "id+")]
    Id,
    #[serde(rename = "id-")]
    IdReverse,
    #[serde(rename = "nickname+")]
    Nickname,
    #[serde(rename = "nickname-")]
    NicknameReverse,
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
    pub role_id: SystemUserRole,
    /// 用户名
    pub username: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRegisterQQParams {
    /// 权限策略
    pub access_policy: Vec<AccessPolicyItemEnum>,
    /// 头像链接
    pub logo: String,
    /// 备注
    pub remark: String,
    /// 角色列表
    pub role_id: SystemUserRole,
    /// 用户名
    pub username: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserUpdateParams {
    /// 用户 ID
    pub id: i64,
    /// 权限策略
    pub access_policy: Option<Vec<AccessPolicyItemEnum>>,
    /// 头像链接
    pub logo: Option<String>,
    /// 昵称
    pub nickname: Option<String>,
    /// 手机号
    pub phone: Option<String>,
    /// QQ
    pub qq: Option<String>,
    /// 备注
    pub remark: Option<String>,
    /// 角色列表
    pub role_id: SystemUserRole,
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
    /// 旧密码
    pub old_password: String,
    /// 备注
    pub remark: String,
    /// 角色列表
    pub role_id: SystemUserRole,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserUpdatePasswordByAdminParams {
    /// 新密码
    pub password: String,
    /// 用户 ID
    pub user_id: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserListParams {
    #[serde(flatten)]
    pub pagination: Pagination,
    /// 昵称
    pub nickname: String,
    /// 角色 ID
    pub role_ids: Option<Vec<SystemUserRole>>,
    /// 排序优先级
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
#[tracing::instrument(skip(auth))]
pub async fn register(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<UserRegisterParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if auth.info.role_id != SystemUserRole::Admin {
        return Ok((StatusCode::FORBIDDEN, "Forbidden".to_string()).into_response());
    }

    Ok(do_register(
        auth,
        payload.access_policy,
        payload.logo,
        payload.remark,
        payload.role_id,
        payload.username,
    )
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
    .into_response())
}

/// 用QQ注册用户
/// POST /user/register/qq
#[tracing::instrument(skip(auth))]
pub async fn register_qq(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<UserRegisterQQParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if auth.info.role_id != SystemUserRole::Admin {
        return Ok((StatusCode::FORBIDDEN, "Forbidden".to_string()).into_response());
    }

    Ok(do_register_qq(
        auth,
        payload.access_policy,
        payload.logo,
        payload.remark,
        payload.role_id,
        payload.username,
    )
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
    .into_response())
}

/// 获取用户信息
/// GET /user/info/{userId}
#[tracing::instrument(skip(auth))]
pub async fn get_info(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(user_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if auth.info.role_id != SystemUserRole::Admin {
        return Ok((StatusCode::FORBIDDEN, "Forbidden".to_string()).into_response());
    }

    Ok(do_get_info(auth, user_id)
        .await
        .map_err(|err| (StatusCode::NOT_FOUND, err.to_string()))?
        .into_response())
}

/// 更新用户信息
/// POST /user/update
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<UserUpdateParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if auth.info.role_id != SystemUserRole::Admin {
        return Ok((StatusCode::FORBIDDEN, "Forbidden".to_string()).into_response());
    }

    Ok(do_update(
        auth,
        payload.id,
        payload.access_policy,
        payload.logo,
        payload.nickname,
        payload.phone,
        payload.qq,
        payload.remark,
        payload.role_id,
    )
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
    .into_response())
}

/// 更新用户密码
/// POST /user/update_password
#[tracing::instrument(skip(auth))]
pub async fn update_password(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<UserUpdatePasswordParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if auth.info.role_id != SystemUserRole::Admin {
        return Ok((StatusCode::FORBIDDEN, "Forbidden".to_string()).into_response());
    }

    Ok(do_update_password(
        auth,
        payload.access_policy,
        payload.id,
        payload.logo,
        payload.old_password,
        payload.remark,
        payload.role_id,
    )
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
    .into_response())
}

/// 更新用户密码（管理员）
/// POST /user/update_password_by_admin
#[tracing::instrument(skip(auth))]
pub async fn update_password_by_admin(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<UserUpdatePasswordByAdminParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if auth.info.role_id != SystemUserRole::Admin {
        return Ok((StatusCode::FORBIDDEN, "Forbidden".to_string()).into_response());
    }

    Ok(
        do_update_password_by_admin(auth, payload.password, payload.user_id)
            .await
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
            .into_response(),
    )
}

/// 删除用户
/// DELETE /user/{workId}
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(work_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if auth.info.role_id != SystemUserRole::Admin {
        return Ok((StatusCode::FORBIDDEN, "Forbidden".to_string()).into_response());
    }

    Ok(do_delete(auth, work_id)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
        .into_response())
}

/// 用户信息(批量查询)
/// POST /user/info/list
#[tracing::instrument(skip(auth))]
pub async fn list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<UserListParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if auth.info.role_id != SystemUserRole::Admin {
        return Ok((StatusCode::FORBIDDEN, "Forbidden".to_string()).into_response());
    }

    Ok(do_list(
        auth,
        payload.pagination,
        payload.nickname,
        payload.role_ids,
        payload
            .sort
            .map(|sorts| sorts.into_iter().map(|s| format!("{:?}", s)).collect()),
        payload.username,
    )
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
    .into_response())
}

/// 踢出用户
/// DELETE /user/kick_out/{workId}
#[tracing::instrument(skip(auth))]
pub async fn kick_out(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(work_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if auth.info.role_id != SystemUserRole::Admin {
        return Ok((StatusCode::FORBIDDEN, "Forbidden".to_string()).into_response());
    }

    Ok(do_kick_out(auth, work_id)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
        .into_response())
}
