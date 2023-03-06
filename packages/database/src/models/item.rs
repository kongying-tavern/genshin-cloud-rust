use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "item", schema_name = "genshin_map")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    pub version: i64,
    pub creator_id: Option<i64>,
    pub updater_id: Option<i64>,
    pub create_time: DateTime,
    pub update_time: Option<DateTime>,
    pub del_flag: i16,

    pub name: String,
    pub area_id: i64,
    pub default_refresh_time: i64,
    pub default_content: Option<String>,
    pub default_count: i32,
    pub icon_tag: String,
    pub icon_style_type: i32,
    pub hidden_flag: i32,
    pub sort_index: i32,
    pub special_flag: Option<i32>,
    pub sync_tag: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
