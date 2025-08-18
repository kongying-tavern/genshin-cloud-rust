use serde::{Deserialize, Serialize};
use std::ops::BitOr;
use strum::EnumIter;

use sea_orm::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum HiddenFlag {
    /// 可见
    #[sea_orm(num_value = 0)]
    Visible = 0,
    /// 隐藏
    #[sea_orm(num_value = 1)]
    Hidden = 1,
    /// 内鬼
    #[sea_orm(num_value = 2)]
    Spy = 2,
    /// 彩蛋
    #[sea_orm(num_value = 3)]
    Superise = 3,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum HistoryEditType {
    /// 未知
    #[sea_orm(num_value = 0)]
    #[default]
    Unknown = 0,
    /// 新增
    #[sea_orm(num_value = 1)]
    Added = 1,
    /// 修改
    #[sea_orm(num_value = 2)]
    Modified = 2,
    /// 删除
    #[sea_orm(num_value = 3)]
    Deleted = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum HistoryOperationType {
    /// 地区
    #[sea_orm(num_value = 1)]
    Area = 1,
    /// 图标
    #[sea_orm(num_value = 2)]
    Icon = 2,
    /// 物品
    #[sea_orm(num_value = 3)]
    Item = 3,
    /// 点位
    #[sea_orm(num_value = 4)]
    Position = 4,
    /// 标签
    #[sea_orm(num_value = 5)]
    Tag = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum ScopeStatType {
    /// 按天统计
    #[sea_orm(string_value = "DAY")]
    DAY,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, EnumIter, DeriveActiveEnum,
)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum IconStyleType {
    /// 默认
    #[sea_orm(num_value = 0)]
    #[default]
    Default = 0,
    /// 无边框
    #[sea_orm(num_value = 1)]
    NoBorder = 1,
    /// 类神瞳
    #[sea_orm(num_value = 3)]
    Oculus = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum MarkerLinkageLinkAction {
    /// 触发
    #[sea_orm(string_value = "TRIGGER")]
    Trigger,
    /// 全部触发
    #[sea_orm(string_value = "TRIGGER_ALL")]
    TriggerAll,
    /// 任意触发
    #[sea_orm(string_value = "TRIGGER_ANY")]
    TriggerAny,
    /// 相关
    #[sea_orm(string_value = "RELATED")]
    Related,
    /// 有向
    #[sea_orm(string_value = "DIRECTED")]
    Directed,
    /// 单向路径
    #[sea_orm(string_value = "PATH_UNI_DIR")]
    PathUniDir,
    /// 双向路径
    #[sea_orm(string_value = "PATH_BI_DIR")]
    PathBiDir,
    /// 等价
    #[sea_orm(string_value = "EQUIVALENT")]
    Equivalent,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum MarkerPunctuateStatus {
    /// 暂存
    #[sea_orm(num_value = 0)]
    Pending = 0,
    /// 审核中
    #[sea_orm(num_value = 1)]
    Reviewing = 1,
    /// 不通过
    #[sea_orm(num_value = 2)]
    Rejected = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum MarkerPunctuateMethodType {
    /// 新增
    #[sea_orm(num_value = 1)]
    Added = 1,
    /// 修改
    #[sea_orm(num_value = 2)]
    Modified = 2,
    /// 删除
    #[sea_orm(num_value = 3)]
    Deleted = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum SystemActionLogAction {
    /// 登录
    #[sea_orm(string_value = "LOGIN")]
    Login,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum SystemUserRole {
    /// 系统管理员
    Admin = 2,
    /// 地图管理员
    MapManager = 3,
    /// 测试打点员
    MapNeigui = 4,
    /// 地图打点员
    MapPunctuate = 5,
    /// 地图用户
    MapUser = 6,
    /// 匿名用户
    Visitor = 100,
}

impl From<i32> for SystemUserRole {
    fn from(v: i32) -> Self {
        match v {
            2 => SystemUserRole::Admin,
            3 => SystemUserRole::MapManager,
            4 => SystemUserRole::MapNeigui,
            5 => SystemUserRole::MapPunctuate,
            6 => SystemUserRole::MapUser,
            100 => SystemUserRole::Visitor,
            x if x < 2 => SystemUserRole::Admin,
            _ => SystemUserRole::Visitor,
        }
    }
}

/// `HiddenFlag` 的掩码枚举，从低到高分别分配一个二进制位，用于配合权限等级的掩码计算
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter)]
#[repr(u8)]
pub enum HiddenFlagMask {
    Visible = 1 << 0,
    Hidden = 1 << 1,
    Spy = 1 << 2,
    Superise = 1 << 3,
}

impl BitOr for HiddenFlagMask {
    type Output = u8;

    fn bitor(self, rhs: Self) -> Self::Output {
        (self as u8) | (rhs as u8)
    }
}

impl BitOr<u8> for HiddenFlagMask {
    type Output = u8;

    fn bitor(self, rhs: u8) -> Self::Output {
        (self as u8) | rhs
    }
}

impl BitOr<HiddenFlagMask> for u8 {
    type Output = u8;

    fn bitor(self, rhs: HiddenFlagMask) -> Self::Output {
        self | (rhs as u8)
    }
}

impl Into<u8> for SystemUserRole {
    fn into(self) -> u8 {
        match self {
            SystemUserRole::Admin => {
                HiddenFlagMask::Visible
                    | HiddenFlagMask::Hidden
                    | HiddenFlagMask::Spy
                    | HiddenFlagMask::Superise
            } // 1|2|4|8 = 15
            SystemUserRole::MapManager => {
                HiddenFlagMask::Visible | HiddenFlagMask::Hidden | HiddenFlagMask::Superise
            } // 1|2|8 = 11
            SystemUserRole::MapNeigui => {
                HiddenFlagMask::Visible
                    | HiddenFlagMask::Hidden
                    | HiddenFlagMask::Spy
                    | HiddenFlagMask::Superise
            } // 1|2|4|8 = 15
            SystemUserRole::MapPunctuate => {
                HiddenFlagMask::Visible | HiddenFlagMask::Hidden | HiddenFlagMask::Superise
            } // 1|2|8 = 11
            SystemUserRole::MapUser => HiddenFlagMask::Visible | HiddenFlagMask::Superise, // 1|8 = 9
            SystemUserRole::Visitor => HiddenFlagMask::Visible | HiddenFlagMask::Superise, // 1|8 = 9
        }
    }
}
