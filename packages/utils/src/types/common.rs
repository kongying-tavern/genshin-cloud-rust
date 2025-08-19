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
