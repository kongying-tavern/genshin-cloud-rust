use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::EnumIter;

use sea_orm::FromJsonQueryResult;
use sea_orm::prelude::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, EnumIter, DeriveActiveEnum,
)]
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

// SeaORM requires JSON column types to implement certain traits like
// `FromJsonQueryResult` / `TryGetableFromJson` / `ValueType`. A plain
// `HashMap<String, String>` does not implement those. Wrap it in a
// newtype and derive `FromJsonQueryResult` so it can be used directly in
// entity models as `#[sea_orm(column_type = "Json")]`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, FromJsonQueryResult)]
pub struct IconURLVariantsWrapper(pub HashMap<String, String>);

// Keep a compatibility alias used by other codepaths in the repo. Prefer
// `IconURLVariantsWrapper` for DB models that require SeaORM traits.
pub type IconURLVariants = IconURLVariantsWrapper;
