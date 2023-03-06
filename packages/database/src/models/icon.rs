use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "icon", schema_name = "genshin_map")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    pub version: i64,
    pub creator_id: Option<i64>,
    pub updater_id: Option<i64>,
    pub create_time: DateTime,
    pub update_time: Option<DateTime>,
    pub del_flag: i16,

    pub icon_id: String,
    pub name: String,
    pub url: String,
    pub creator: String,
    pub sync_tag: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
