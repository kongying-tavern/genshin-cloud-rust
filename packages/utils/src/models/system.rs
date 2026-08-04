use serde::{Deserialize, Serialize};

use crate::types::{AccessPolicyList, SystemUserRole};

/// 角色列表 VO（Java `SysRoleVo`；前端按 `roleId`（数字）查表构建权限掩码，
/// 序列化为字符串会断掉整条数据初始化链）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysRoleVo {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub sort: i64,
}

/// `roleId` 按 Java 契约序列化为**数字**（前端 `roleMap.get(roleId)` 用数字 id 匹配）。
fn serialize_role_id<S>(role: &SystemUserRole, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_i32(*role as i32)
}

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

    /// 角色（序列化为数字 id，Java 契约）
    #[serde(serialize_with = "serialize_role_id")]
    pub role_id: SystemUserRole,
    /// 权限策略
    pub access_policy: AccessPolicyList,
    /// 备注
    pub remark: Option<String>,
}
