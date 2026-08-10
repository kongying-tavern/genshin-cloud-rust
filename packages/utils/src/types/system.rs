use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter, EnumString};

use sea_orm::{FromJsonQueryResult, prelude::*};

use super::HiddenFlag;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum SystemActionLogAction {
    /// 登录
    #[sea_orm(string_value = "LOGIN")]
    Login,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct SysActionLogExtra {
    pub access_paths: Vec<AccessPolicyItem>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, FromJsonQueryResult)]
pub struct AccessPolicyList(pub Vec<AccessPolicyItemEnum>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(rename_all = "camelCase")]
pub struct AccessPolicyItem {
    pub passed: bool,
    pub policy: AccessPolicyItemEnum,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, EnumIter, AsRefStr, EnumString, Display)]
pub enum AccessPolicyItemEnum {
    /// 与最后一次登录 IP 相同
    #[strum(serialize = "ip:same_last_ip")]
    #[serde(rename = "ip:same_last_ip")]
    IpSameLastIp,
    /// 对列表中有效的 IP 放行
    #[strum(serialize = "ip:pass_allow_ip")]
    #[serde(rename = "ip:pass_allow_ip")]
    IpPassAllowIp,
    /// 对列表中禁用的 IP 拦截
    #[strum(serialize = "ip:block_disallow_ip")]
    #[serde(rename = "ip:block_disallow_ip")]
    IpBlockDisallowIp,
    /// 与最后一次登录地区相同
    #[strum(serialize = "ip:same_last_region")]
    #[serde(rename = "ip:same_last_region")]
    IpSameLastRegion,
    /// 对列表中有效的地区放行
    #[strum(serialize = "ip:pass_allow_region")]
    #[serde(rename = "ip:pass_allow_region")]
    IpPassAllowRegion,
    /// 对列表中禁用的地区拦截
    #[strum(serialize = "ip:block_disallow_region")]
    #[serde(rename = "ip:block_disallow_region")]
    IpBlockDisallowRegion,
    /// 与最后一次登录设备相同
    #[strum(serialize = "dev:same_last_device")]
    #[serde(rename = "dev:same_last_device")]
    DevSameLastDevice,
    /// 对列表中有效的设备放行
    #[strum(serialize = "dev:pass_allow_device")]
    #[serde(rename = "dev:pass_allow_device")]
    DevPassAllowDevice,
    /// 对列表中禁用的设备拦截
    #[strum(serialize = "dev:block_disallow_device")]
    #[serde(rename = "dev:block_disallow_device")]
    DevBlockDisallowDevice,
}

/// 反序列化兼容历史/远程数据：`sys_user.access_policy` / `sys_user_invitation.access_policy`
/// 中可能存无前缀格式（如 `same_last_ip`），与带前缀的 Java 契约（`ip:same_last_ip`）并存。
/// 先去掉 `ip:` / `dev:` / `area:` 前缀再匹配变体；序列化仍输出带前缀格式（Java 契约）。
impl<'de> Deserialize<'de> for AccessPolicyItemEnum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bare = s
            .strip_prefix("ip:")
            .or_else(|| s.strip_prefix("dev:"))
            .or_else(|| s.strip_prefix("area:"))
            .unwrap_or(s.as_str());
        match bare {
            "same_last_ip" => Ok(Self::IpSameLastIp),
            "pass_allow_ip" => Ok(Self::IpPassAllowIp),
            "block_disallow_ip" => Ok(Self::IpBlockDisallowIp),
            "same_last_region" => Ok(Self::IpSameLastRegion),
            "pass_allow_region" => Ok(Self::IpPassAllowRegion),
            "block_disallow_region" => Ok(Self::IpBlockDisallowRegion),
            "same_last_device" => Ok(Self::DevSameLastDevice),
            "pass_allow_device" => Ok(Self::DevPassAllowDevice),
            "block_disallow_device" => Ok(Self::DevBlockDisallowDevice),
            _ => Err(serde::de::Error::custom(format!(
                "unknown access policy item: {s}"
            ))),
        }
    }
}

/// Sort keys for the user list. The serde renames are the **wire contract**
/// ("createTime+", "createTime-", "id+", "id-", "nickname+", "nickname-") —
/// the business layer matches the enum variants directly, so renaming a
/// variant is a compile error instead of a silently-ignored sort key.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter)]
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

/// Sort keys for the user device list（wire 契约与 UserSort 一致：字段+ 升序 / 字段- 降序）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter)]
pub enum DeviceSort {
    #[serde(rename = "deviceId+")]
    DeviceId,
    #[serde(rename = "deviceId-")]
    DeviceIdReverse,
    #[serde(rename = "id+")]
    Id,
    #[serde(rename = "id-")]
    IdReverse,
    #[serde(rename = "ipv4+")]
    Ipv4,
    #[serde(rename = "ipv4-")]
    Ipv4Reverse,
    #[serde(rename = "lastLoginTime+")]
    LastLoginTime,
    #[serde(rename = "lastLoginTime-")]
    LastLoginTimeReverse,
    #[serde(rename = "status+")]
    Status,
    #[serde(rename = "status-")]
    StatusReverse,
    #[serde(rename = "updateTime+")]
    UpdateTime,
    #[serde(rename = "updateTime-")]
    UpdateTimeReverse,
}

/// Sort keys for the user invitation list（wire 契约：createTime± / id± / updateTime± / username±）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter)]
pub enum InvitationSort {
    #[serde(rename = "createTime+")]
    CreateTime,
    #[serde(rename = "createTime-")]
    CreateTimeReverse,
    #[serde(rename = "id+")]
    Id,
    #[serde(rename = "id-")]
    IdReverse,
    #[serde(rename = "updateTime+")]
    UpdateTime,
    #[serde(rename = "updateTime-")]
    UpdateTimeReverse,
    #[serde(rename = "username+")]
    Username,
    #[serde(rename = "username-")]
    UsernameReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum SystemUserRole {
    /// 系统管理员
    Admin = 0,
    /// 地图管理员
    MapManager = 1,
    /// 测试打点员
    MapNeigui = 2,
    /// 地图打点员
    MapPunctuate = 3,
    /// 地图用户
    MapUser = 4,
    /// 匿名用户
    Visitor = 5,
}

/// 序列化为**数字**（Java 契约；前端 `roleId` 为数字枚举值）。
impl Serialize for SystemUserRole {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i32(*self as i32)
    }
}

/// 反序列化同时接受数字（新契约/前端）与枚举名（旧数据）。
impl<'de> Deserialize<'de> for SystemUserRole {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
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
                _ => Err(serde::de::Error::custom(format!("unknown roleId {n}"))),
            },
            serde_json::Value::String(s) => match s.as_str() {
                "Admin" => Ok(SystemUserRole::Admin),
                "MapManager" => Ok(SystemUserRole::MapManager),
                "MapNeigui" => Ok(SystemUserRole::MapNeigui),
                "MapPunctuate" => Ok(SystemUserRole::MapPunctuate),
                "MapUser" => Ok(SystemUserRole::MapUser),
                "Visitor" => Ok(SystemUserRole::Visitor),
                _ => Err(serde::de::Error::custom(format!("unknown roleId {s}"))),
            },
            _ => Err(serde::de::Error::custom(
                "roleId must be an integer or enum name",
            )),
        }
    }
}

impl SystemUserRole {
    /// Check whether this role may access content tagged with `flag`.
    /// Kept for the role-gated query layer (RBAC); not yet wired into every route.
    #[allow(dead_code)]
    fn is_available(self, flag: HiddenFlag) -> bool {
        if matches!(flag, HiddenFlag::Visible | HiddenFlag::Suprise) {
            return true;
        }

        match self {
            SystemUserRole::Admin | SystemUserRole::MapNeigui => {
                matches!(flag, HiddenFlag::Hidden | HiddenFlag::Spy)
            },
            SystemUserRole::MapManager | SystemUserRole::MapPunctuate => {
                matches!(flag, HiddenFlag::Hidden)
            },
            SystemUserRole::MapUser | SystemUserRole::Visitor => false,
        }
    }
}
