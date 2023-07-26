use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "icon_type_link")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(version)]
    pub version: i64,
    #[sea_orm(create_time)]
    pub create_time: DateTime,
    #[sea_orm(update_time)]
    pub update_time: Option<DateTime>,

    pub creator_id: Option<i64>,
    pub updater_id: Option<i64>,
    pub del_flag: i16,

    pub type_id: i64,
    pub icon_id: i64,
    pub sync_tag: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
