use serde::{Deserialize, Serialize};
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
    Suprise = 3,
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
    fn is_available(self, flag: HiddenFlag) -> bool {
        if matches!(flag, HiddenFlag::Visible | HiddenFlag::Suprise) {
            return true;
        }

        match self {
            SystemUserRole::Admin | SystemUserRole::MapNeigui => {
                matches!(flag, HiddenFlag::Hidden | HiddenFlag::Spy)
            }
            SystemUserRole::MapManager | SystemUserRole::MapPunctuate => {
                matches!(flag, HiddenFlag::Hidden)
            }
            SystemUserRole::MapUser | SystemUserRole::Visitor => false,
        }
    }
}
