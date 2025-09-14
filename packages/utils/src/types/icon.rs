use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::EnumIter;

use sea_orm::prelude::*;
use sea_orm::FromJsonQueryResult;

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
