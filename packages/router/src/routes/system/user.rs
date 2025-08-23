use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::{
    UserListParams, UserRegisterParams, UserRegisterQQParams, UserUpdateParams,
    UserUpdatePasswordByAdminParams, UserUpdatePasswordParams,
};

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
    Json(payload): Json<UserRegisterQQParams>,
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
