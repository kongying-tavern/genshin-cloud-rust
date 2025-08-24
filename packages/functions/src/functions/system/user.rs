use anyhow::Result;

use _utils::{
    jwt::AuthInfo,
    models::Pagination,
    types::{AccessPolicyItemEnum, SystemUserRole},
};

// 业务处理函数
pub async fn do_register(
    _auth: AuthInfo,
    _access_policy: Vec<AccessPolicyItemEnum>,
    _logo: String,
    _remark: String,
    _role_id: SystemUserRole,
    _username: String,
) -> Result<()> {
    // TODO: 实现注册逻辑
    Ok(())
}

pub async fn do_register_qq(
    _auth: AuthInfo,
    _access_policy: Vec<AccessPolicyItemEnum>,
    _logo: String,
    _remark: String,
    _role_id: SystemUserRole,
    _username: String,
) -> Result<()> {
    // TODO: 实现QQ注册逻辑
    Ok(())
}

pub async fn do_get_info(_auth: AuthInfo, _user_id: i64) -> Result<()> {
    // TODO: 实现获取用户信息逻辑
    Ok(())
}

pub async fn do_update(
    _auth: AuthInfo,
    _id: i64,
    _access_policy: Option<Vec<AccessPolicyItemEnum>>,
    _logo: Option<String>,
    _nickname: Option<String>,
    _phone: Option<String>,
    _qq: Option<String>,
    _remark: Option<String>,
    _role_id: SystemUserRole,
) -> Result<()> {
    // TODO: 实现更新用户信息逻辑
    Ok(())
}

pub async fn do_update_password(
    _auth: AuthInfo,
    _access_policy: Vec<AccessPolicyItemEnum>,
    _id: i64,
    _logo: String,
    _old_password: String,
    _remark: String,
    _role_id: SystemUserRole,
) -> Result<()> {
    // TODO: 实现更新用户密码逻辑
    Ok(())
}

pub async fn do_update_password_by_admin(
    _auth: AuthInfo,
    _password: String,
    _user_id: i64,
) -> Result<()> {
    // TODO: 实现管理员更新用户密码逻辑
    Ok(())
}

pub async fn do_delete(_auth: AuthInfo, _work_id: i64) -> Result<()> {
    // TODO: 实现删除用户逻辑
    Ok(())
}

pub async fn do_list(
    _auth: AuthInfo,
    _pagination: Pagination,
    _nickname: String,
    _role_ids: Option<Vec<SystemUserRole>>,
    _sort: Option<Vec<String>>, // 简化为 String，因为 UserSort 已移动
    _username: String,
) -> Result<()> {
    // TODO: 实现批量查询用户信息逻辑
    Ok(())
}

pub async fn do_kick_out(_auth: AuthInfo, _work_id: String) -> Result<()> {
    // TODO: 实现踢出用户逻辑
    Ok(())
}
