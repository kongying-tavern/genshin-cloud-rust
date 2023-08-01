use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "area")]
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

    pub name: String,
    pub code: Option<String>,
    pub content: Option<String>,
    pub icon_tag: String,
    pub parent_id: i64,
    pub is_final: bool,
    pub hidden_flag: i32,
    pub sort_index: i32,
    pub sync_tag: Option<String>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            id: Default::default(),
            version: Default::default(),
            create_time: Default::default(),
            update_time: Default::default(),
            creator_id: Default::default(),
            updater_id: Default::default(),
            del_flag: Default::default(),
            name: Default::default(),
            code: Default::default(),
            content: Default::default(),
            icon_tag: Default::default(),
            parent_id: Default::default(),
            is_final: Default::default(),
            hidden_flag: Default::default(),
            sort_index: Default::default(),
            sync_tag: Default::default(),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// PO 与 DTO 之间的转换逻辑

// TODO - 完善所有 PO 到 DTO 之间的数据转换，顺带补一下 PO 的 Default trait
use _utils::schemas::area::Schema as DTO;

impl From<DTO> for Model {
    fn from(info: DTO) -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl From<Model> for DTO {
    fn from(model: Model) -> Self {
        Self {
            ..Default::default()
        }
    }
}
