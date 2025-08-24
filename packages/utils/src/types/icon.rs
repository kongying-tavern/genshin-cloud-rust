use serde::{Deserialize, Serialize};
use strum::EnumIter;

use sea_orm::prelude::*;

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
    #[sea_orm(num_value = 2)]
    #[deprecated(note = "该样式已废弃，请使用 Oculus")]
    LikeOculus = 2,
    /// 类神瞳无对勾
    #[sea_orm(num_value = 3)]
    Oculus = 3,
}
