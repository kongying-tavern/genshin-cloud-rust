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
            Some(2) => Ok(SystemUserRole::MapBeta),
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

/// 存档 VO（对齐前端 `SysArchiveVo` 契约；`time` 为毫秒时间戳，Java `Timestamp` NUMBER_INT）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysArchiveVo {
    /// 存档时间
    pub time: f64,
    /// 存档（存档 JSON 文本）
    pub archive: String,
    /// 存档历史下标
    pub history_index: i64,
}

/// 存档槽位 VO（对齐前端 `SysArchiveSlotVo` 契约；时间字段为毫秒时间戳）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysArchiveSlotVo {
    /// 版本号
    pub version: i64,
    /// 存档 ID
    pub id: i64,
    /// 存档名称
    pub name: Option<String>,
    /// 槽位顺序
    pub slot_index: i32,
    /// 创建时间
    pub create_time: f64,
    /// 更新时间
    pub update_time: Option<f64>,
    /// 存档列表
    pub archive: Vec<SysArchiveVo>,
}

/// 存档槽位 VO（列表接口返回结构，`archive` 为存档 JSON 文本）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveSlotVo {
    /// 槽位顺序
    pub slot_index: i32,
    /// 存档时间（毫秒时间戳）
    pub time: f64,
    /// 存档（存档 JSON 文本）
    pub archive: String,
}

/// 用户设备 VO（对齐前端 `SysUserDeviceVo`；时间字段为毫秒时间戳）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysUserDeviceVo {
    /// ID
    pub id: i64,
    /// 创建时间
    pub create_time: f64,
    /// 更新时间
    pub update_time: Option<f64>,
    /// 用户 ID
    pub user_id: Option<i64>,
    /// 设备标识
    pub device_id: String,
    /// IPv4
    pub ipv4: Option<String>,
    /// 设备状态
    pub status: i32,
    /// 上次登录时间
    pub last_login_time: Option<f64>,
}

/// 操作日志 VO（对齐前端 `SysActionLogVo`；时间字段为毫秒时间戳）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysActionLogVo {
    /// ID
    pub id: i64,
    /// 创建时间
    pub create_time: f64,
    /// 更新时间
    pub update_time: Option<f64>,
    /// 用户 ID
    pub user_id: Option<i64>,
    /// IPv4
    pub ipv4: Option<String>,
    /// 设备标识
    pub device_id: String,
    /// 动作（如 "LOGIN"）
    pub action: String,
    /// 是否错误
    pub is_error: bool,
    /// 附加信息
    pub extra_data: Option<serde_json::Value>,
}

/// 用户邀请 VO（对齐前端 `SysUserInvitationVo`；时间字段为毫秒时间戳）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysUserInvitationVo {
    /// ID
    pub id: i64,
    /// 创建时间
    pub create_time: f64,
    /// 更新时间
    pub update_time: Option<f64>,
    /// 创建人 ID
    pub creator_id: Option<i64>,
    /// 邀请码
    pub code: String,
    /// 用户名
    pub username: String,
    /// 角色 ID（数字）
    pub role_id: Option<i64>,
    /// 备注
    pub remark: Option<String>,
    /// 权限策略
    pub access_policy: Option<serde_json::Value>,
}

/// 用户邀请精简 VO（对齐 Java `SysUserInvitationSmallVo`，
/// `updateInvitation` 的返回体：把最终邀请码回给管理端展示/复制）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysUserInvitationSmallVo {
    /// 邀请码
    pub code: String,
    /// 用户名
    pub username: String,
}
