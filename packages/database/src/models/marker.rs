use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "marker")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,

    pub version: i64,

    pub creator_id: Option<i64>,
    pub updater_id: Option<i64>,
    pub create_time: DateTime,
    pub update_time: Option<DateTime>,

    pub marker_stamp: Option<String>,
    pub marker_title: Option<String>,
    pub position: String,
    pub content: String,
    pub picture: Option<String>,
    pub marker_creator_id: i64,
    pub picture_creator_id: Option<i64>,
    pub video_path: Option<String>,
    pub refresh_time: i64,
    pub hidden_flag: i32,
    pub sync_tag: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
