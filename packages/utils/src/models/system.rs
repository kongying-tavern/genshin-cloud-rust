use serde::{Deserialize, Serialize};

use crate::{
    models::Pagination,
    types::{AccessPolicyItemEnum, AccessPolicyList},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysUserVO {
    /// ID
    pub id: u64,

    /// 用户名
    pub username: String,
    /// 昵称
    pub nickname: Option<String>,
    /// QQ
    pub qq: Option<String>,
    /// 手机号
    pub phone: Option<String>,
    /// 头像链接
    pub logo: Option<String>,

    /// 权限策略
    pub access_policy: AccessPolicyList,
    /// 备注
    pub remark: Option<String>,
}

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
    pub role_id: i64,
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
    pub role_id: i64,
    /// 用户名
    pub username: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserUpdateParams {
    /// 用户ID
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
    pub role_id: i64,
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
