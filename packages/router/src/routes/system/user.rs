use anyhow::Result;
use serde::{Deserialize, Serialize};

use axum::{
    extract::{Json, Path},
    response::IntoResponse,
};

use crate::middlewares::{ExtractAdmin, ExtractAuthInfo, ExtractManager};
use _functions::functions::system::user::*;
use _utils::{models::Pagination, types::AccessPolicyItemEnum, types::SystemUserRole};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRegisterParams {
    /// 权限策略
    pub access_policy: Option<Vec<AccessPolicyItemEnum>>,
    /// 头像链接
    pub logo: Option<String>,
    /// 备注
    pub remark: Option<String>,
    /// 角色列表
    pub role_id: Option<SystemUserRole>,
    /// 用户名
    pub username: String,
    /// 初始密码
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRegisterQQParams {
    /// 权限策略
    pub access_policy: Option<Vec<AccessPolicyItemEnum>>,
    /// 头像链接
    pub logo: Option<String>,
    /// 备注
    pub remark: Option<String>,
    /// 用户名（QQ 号）
    pub username: String,
    /// 初始密码
    pub password: String,
    /// QQ 号（与 Java 契约一致：注册时 username 即 QQ 号，缺省时与
    /// username 相同；不做腾讯 OAuth 的 openid 语义）
    pub qq: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserUpdateParams {
    /// 用户 ID
    pub user_id: Option<i64>,
    /// 用户 ID（前端 InfoEditor 展开 userStore.info 时使用 `id` 字段）
    pub id: Option<i64>,
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
    pub role_id: Option<SystemUserRole>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserUpdatePasswordParams {
    /// 权限策略
    pub access_policy: Option<Vec<AccessPolicyItemEnum>>,
    /// ID
    pub user_id: i64,
    /// 头像链接
    pub logo: Option<String>,
    /// 旧密码
    pub old_password: String,
    /// 新密码
    pub new_password: Option<String>,
    /// 新密码（前端兼容字段，与 new_password 二选一）
    pub password: Option<String>,
    /// 备注
    pub remark: Option<String>,
    /// 角色列表
    pub role_id: Option<SystemUserRole>,
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
    pub nickname: Option<String>,
    /// 角色 ID
    pub role_ids: Option<Vec<SystemUserRole>>,
    /// 排序优先级
    pub sort: Option<Vec<_utils::types::UserSort>>,
    /// 用户名
    pub username: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // request body type for the kick-out endpoint (not yet wired)
pub struct UserKickOutParams {
    pub work_id: String,
}

/// 注册用户（地图管理员及以上，Java authorities-filter 将
/// /system/user/register 划给 MAP_MANAGER）
/// POST /user/register
#[tracing::instrument(skip(auth, payload))]
pub async fn register(
    ExtractManager(auth): ExtractManager,
    Json(payload): Json<UserRegisterParams>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    Ok(Json(
        do_register(
            auth,
            payload.access_policy,
            payload.logo,
            payload.remark,
            payload.username,
            payload.password,
        )
        .await
        .map_err(crate::routes::internal_error)?,
    )
    .into_response())
}

/// 用QQ注册用户（公开接口：QQ 授权后未登录调用）
/// POST /user/register/qq
#[tracing::instrument(skip(payload))]
pub async fn register_qq(
    Json(payload): Json<UserRegisterQQParams>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    Ok(Json(
        do_register_qq(
            payload.access_policy,
            payload.logo,
            payload.remark,
            payload.username,
            payload.password,
            payload.qq,
        )
        .await
        .map_err(crate::routes::internal_error)?,
    )
    .into_response())
}

/// 获取用户信息
/// GET /user/info/{userId}
#[tracing::instrument(skip(auth))]
pub async fn get_info(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(user_id): Path<i64>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    Ok(Json(_utils::models::wrapper::CommonResponse::new(
        do_get_info(auth, user_id).await,
    ))
    .into_response())
}

/// 更新用户信息
/// POST /user/update
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<UserUpdateParams>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    let uid = payload
        .user_id
        .or(payload.id)
        .ok_or_else(|| crate::routes::route_error("user id is required"))?;
    Ok(Json(
        do_update(
            auth,
            uid,
            payload.access_policy,
            payload.logo,
            payload.nickname,
            payload.phone,
            payload.qq,
            payload.remark,
            payload.role_id,
        )
        .await
        .map_err(|(code, msg)| crate::routes::status_error(code, msg))?,
    )
    .into_response())
}

/// 更新用户密码
/// POST /user/update_password
#[tracing::instrument(skip(auth, payload))]
pub async fn update_password(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<UserUpdatePasswordParams>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    let new_pw = payload.new_password.or(payload.password);
    let Some(new_pw) = new_pw else {
        return Err(crate::routes::route_error("new password is required"));
    };
    Ok(Json(
        do_update_password(auth, payload.user_id, payload.old_password, new_pw)
            .await
            .map_err(|(code, msg)| crate::routes::status_error(code, msg))?,
    )
    .into_response())
}

/// 更新用户密码（管理员）
/// POST /user/update_password_by_admin
#[tracing::instrument(skip(auth, payload))]
pub async fn update_password_by_admin(
    ExtractAdmin(auth): ExtractAdmin,
    Json(payload): Json<UserUpdatePasswordByAdminParams>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    Ok(Json(
        do_update_password_by_admin(auth, payload.password, payload.user_id)
            .await
            .map_err(crate::routes::internal_error)?,
    )
    .into_response())
}

/// 删除用户
/// DELETE /user/{workId}
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAdmin(auth): ExtractAdmin,
    Path(work_id): Path<i64>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    do_delete(auth, work_id)
        .await
        .map_err(crate::routes::internal_error)?;
    Ok(Json(_utils::models::wrapper::CommonResponse::new(Ok(true))).into_response())
}

/// 用户信息(批量查询)
/// POST /user/info/list
/// Java authorities-filter 将本端点划给 MAP_MANAGER
#[tracing::instrument(skip(auth))]
pub async fn list(
    ExtractManager(auth): ExtractManager,
    Json(payload): Json<UserListParams>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    Ok(Json(
        do_list(
            auth,
            payload.pagination,
            payload.nickname,
            payload.role_ids,
            payload.sort,
            payload.username,
        )
        .await
        .map_err(crate::routes::internal_error)?,
    )
    .into_response())
}

/// 踢出用户
/// DELETE /user/kick_out/{workId}
#[tracing::instrument(skip(auth))]
pub async fn kick_out(
    ExtractAdmin(auth): ExtractAdmin,
    Path(work_id): Path<String>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    do_kick_out(auth, work_id)
        .await
        .map_err(crate::routes::internal_error)?;
    Ok(Json(_utils::models::wrapper::CommonResponse::new(Ok(true))).into_response())
}
