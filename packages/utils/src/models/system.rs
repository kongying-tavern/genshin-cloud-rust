use serde::{Deserialize, Serialize};

use crate::types::{AccessPolicyList, SystemUserRole};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysUserVO {
    /// ID
    pub id: i64,
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

    /// 角色
    pub role_id: SystemUserRole,
    /// 权限策略
    pub access_policy: AccessPolicyList,
    /// 备注
    pub remark: Option<String>,
}
