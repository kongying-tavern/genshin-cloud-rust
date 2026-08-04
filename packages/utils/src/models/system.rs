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

/// 反序列化同时接受数字（新契约）与枚举名（旧 Redis payload / 旧客户端）。
fn deserialize_role_id<'de, D>(deserializer: D) -> Result<SystemUserRole, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = serde_json::Value::deserialize(deserializer)?;
    match v {
        serde_json::Value::Number(n) => match n.as_i64() {
            Some(0) => Ok(SystemUserRole::Admin),
            Some(1) => Ok(SystemUserRole::MapManager),
            Some(2) => Ok(SystemUserRole::MapNeigui),
            Some(3) => Ok(SystemUserRole::MapPunctuate),
            Some(4) => Ok(SystemUserRole::MapUser),
            Some(5) => Ok(SystemUserRole::Visitor),
            _ => Err(serde::de::Error::custom(format!("unknown role id {n}"))),
        },
        serde_json::Value::String(s) => SystemUserRole::deserialize(serde_json::Value::String(s))
            .map_err(serde::de::Error::custom),
        _ => Err(serde::de::Error::custom(
            "roleId must be an integer or enum name",
        )),
    }
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

    /// 角色（序列化为数字 id，Java 契约；反序列化兼容数字与枚举名）
    #[serde(
        serialize_with = "serialize_role_id",
        deserialize_with = "deserialize_role_id"
    )]
    pub role_id: SystemUserRole,
    /// 权限策略
    pub access_policy: AccessPolicyList,
    /// 备注
    pub remark: Option<String>,
}
