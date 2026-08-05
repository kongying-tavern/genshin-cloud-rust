use serde::{Deserialize, Serialize};
use strum::EnumIter;

use sea_orm::prelude::*;

/// 序列化为**数字**（Java 契约）——前端 `checkHiddenFlag` 做 `1 << hiddenFlag`
/// 位掩码运算，枚举名字符串（"Visible"）会变成 NaN 导致全部被过滤。
impl Serialize for HiddenFlag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i32(*self as i32)
    }
}

/// 反序列化同时接受数字（新契约/前端）与枚举名（旧数据）。
impl<'de> Deserialize<'de> for HiddenFlag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = serde_json::Value::deserialize(deserializer)?;
        match v {
            serde_json::Value::Number(n) => match n.as_i64() {
                Some(0) => Ok(HiddenFlag::Visible),
                Some(1) => Ok(HiddenFlag::Hidden),
                Some(2) => Ok(HiddenFlag::Spy),
                Some(3) => Ok(HiddenFlag::Suprise),
                _ => Err(serde::de::Error::custom(format!("unknown hiddenFlag {n}"))),
            },
            serde_json::Value::String(s) => match s.as_str() {
                "Visible" => Ok(HiddenFlag::Visible),
                "Hidden" => Ok(HiddenFlag::Hidden),
                "Spy" => Ok(HiddenFlag::Spy),
                "Suprise" => Ok(HiddenFlag::Suprise),
                _ => Err(serde::de::Error::custom(format!("unknown hiddenFlag {s}"))),
            },
            _ => Err(serde::de::Error::custom(
                "hiddenFlag must be an integer or enum name",
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
/// 权限可见层级
pub enum HiddenFlag {
    /// 可见
    #[sea_orm(num_value = 0)]
    Visible = 0,
    /// 隐藏
    #[sea_orm(num_value = 1)]
    Hidden = 1,
    /// 内鬼 / 测试服
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
    /// 初始化（历史数据导入）
    #[sea_orm(num_value = 10)]
    Initialized = 10,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum ScopeStatType {
    /// 按天统计
    #[sea_orm(string_value = "DAY")]
    DAY,
}
