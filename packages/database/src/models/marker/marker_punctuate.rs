use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "marker_punctuate")]
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
    #[sea_orm(default_value = 0)]
    pub del_flag: i16,

    pub punctuate_id: i64,
    pub original_marker_id: Option<i64>,
    pub marker_title: Option<String>,
    pub item_list: String,
    pub position: String,
    pub content: String,
    pub picture: Option<String>,
    pub marker_creator_id: i64,
    pub picture_creator_id: Option<i64>,
    pub video_path: Option<String>,
    pub author: i64,
    pub status: i32,
    pub audit_remark: Option<String>,
    pub method_type: i32,
    pub refresh_time: i64,
    pub hidden_flag: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
