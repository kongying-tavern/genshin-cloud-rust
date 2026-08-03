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

#[derive(
    Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter, AsRefStr, EnumString, Display,
)]
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
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
