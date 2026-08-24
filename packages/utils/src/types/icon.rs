use serde::{Deserialize, Serialize};
use strum::EnumIter;

use sea_orm::prelude::*;

/// 序列化为**数字**（Java 契约）——前端 `iconStyleType` 为 0/1/2/3，
/// 枚举名字符串（"Default"）会导致前端样式判断全部失效。
impl Serialize for IconStyleType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i32(*self as i32)
    }
}

/// 反序列化同时接受数字（新契约/前端）与枚举名（旧数据）。
impl<'de> Deserialize<'de> for IconStyleType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = serde_json::Value::deserialize(deserializer)?;
        match v {
            serde_json::Value::Number(n) => match n.as_i64() {
                Some(0) => Ok(IconStyleType::Default),
                Some(1) => Ok(IconStyleType::NoBorder),
                Some(2) => Ok(IconStyleType::LikeOculus),
                Some(3) => Ok(IconStyleType::Oculus),
                _ => Err(serde::de::Error::custom(format!(
                    "unknown iconStyleType {n}"
                ))),
            },
            serde_json::Value::String(s) => match s.as_str() {
                "Default" => Ok(IconStyleType::Default),
                "NoBorder" => Ok(IconStyleType::NoBorder),
                "LikeOculus" => Ok(IconStyleType::LikeOculus),
                "Oculus" => Ok(IconStyleType::Oculus),
                _ => Err(serde::de::Error::custom(format!(
                    "unknown iconStyleType {s}"
                ))),
            },
            _ => Err(serde::de::Error::custom(
                "iconStyleType must be an integer or enum name",
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
#[allow(deprecated)] // LikeOculus is kept for legacy DB rows (num_value = 2)
pub enum IconStyleType {
    /// 默认
    #[sea_orm(num_value = 0)]
    #[default]
    Default = 0,
    /// 无边框
    #[sea_orm(num_value = 1)]
    NoBorder = 1,
    /// 类神瞳（已废弃，新数据请使用 Oculus；此变体保留以兼容数据库中
    /// num_value = 2 的历史记录，sea-orm 的 DeriveActiveEnum 派生需要它存在）
    #[sea_orm(num_value = 2)]
    LikeOculus = 2,
    /// 类神瞳无对勾
    #[sea_orm(num_value = 3)]
    Oculus = 3,
}
