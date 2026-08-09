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

#[derive(Debug, Clone, Copy, PartialEq, Default, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
/// 权限可见层级
pub enum HiddenFlag {
    /// 可见
    #[sea_orm(num_value = 0)]
    #[default]
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

#[derive(Debug, Clone, Copy, PartialEq, Default, EnumIter, DeriveActiveEnum)]
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

/// 序列化为**数字**（Java 契约）。
impl Serialize for HistoryEditType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i32(*self as i32)
    }
}

/// 反序列化同时接受数字、数字字符串（'0'|'1'|... 前端 editType）与枚举名。
impl<'de> Deserialize<'de> for HistoryEditType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = serde_json::Value::deserialize(deserializer)?;
        match v {
            serde_json::Value::Number(n) => match n.as_i64() {
                Some(0) => Ok(HistoryEditType::Unknown),
                Some(1) => Ok(HistoryEditType::Added),
                Some(2) => Ok(HistoryEditType::Modified),
                Some(3) => Ok(HistoryEditType::Deleted),
                Some(10) => Ok(HistoryEditType::Initialized),
                _ => Err(serde::de::Error::custom(format!("unknown editType {n}"))),
            },
            serde_json::Value::String(s) => match s.as_str() {
                "0" | "Unknown" => Ok(HistoryEditType::Unknown),
                "1" | "Added" => Ok(HistoryEditType::Added),
                "2" | "Modified" => Ok(HistoryEditType::Modified),
                "3" | "Deleted" => Ok(HistoryEditType::Deleted),
                "10" | "Initialized" => Ok(HistoryEditType::Initialized),
                _ => Err(serde::de::Error::custom(format!("unknown editType {s}"))),
            },
            _ => Err(serde::de::Error::custom(
                "editType must be an integer, numeric string or enum name",
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, EnumIter, DeriveActiveEnum)]
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

/// 序列化为**数字**（Java 契约；前端 `type: 4|3` 数字消费）。
impl Serialize for HistoryOperationType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i32(*self as i32)
    }
}

/// 反序列化同时接受数字（新契约/前端）与枚举名（旧数据）。
impl<'de> Deserialize<'de> for HistoryOperationType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = serde_json::Value::deserialize(deserializer)?;
        match v {
            serde_json::Value::Number(n) => match n.as_i64() {
                Some(1) => Ok(HistoryOperationType::Area),
                Some(2) => Ok(HistoryOperationType::Icon),
                Some(3) => Ok(HistoryOperationType::Item),
                Some(4) => Ok(HistoryOperationType::Position),
                _ => Err(serde::de::Error::custom(format!(
                    "unknown operationType {n}"
                ))),
            },
            serde_json::Value::String(s) => match s.as_str() {
                "Area" => Ok(HistoryOperationType::Area),
                "Icon" => Ok(HistoryOperationType::Icon),
                "Item" => Ok(HistoryOperationType::Item),
                "Position" => Ok(HistoryOperationType::Position),
                _ => Err(serde::de::Error::custom(format!(
                    "unknown operationType {s}"
                ))),
            },
            _ => Err(serde::de::Error::custom(
                "operationType must be an integer or enum name",
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum ScopeStatType {
    /// 按天统计
    #[sea_orm(string_value = "DAY")]
    DAY,
}
