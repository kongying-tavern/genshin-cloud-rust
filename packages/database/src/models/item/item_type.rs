use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use _utils::types::enums::HiddenFlag;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "item_type", schema_name = "genshin_map")]
pub struct Model {
    /// 乐观锁
    pub version: i64,
    /// ID
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 创建时间
    pub create_time: DateTime,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 创建人
    pub creator_id: Option<i64>,
    /// 更新人
    pub updater_id: Option<i64>,
    /// 逻辑删除
    pub del_flag: bool,

    /// 图标标签
    pub icon_tag: String,
    /// 类型名
    pub name: String,
    /// 类型补充说明
    pub content: Option<String>,
    /// 父级类型 ID
    /// 无父级则为 -1
    #[sea_orm(indexed)]
    pub parent_id: i64,
    /// 是否为末端类型
    pub is_final: bool,
    /// 权限屏蔽标记
    #[sea_orm(indexed)]
    pub hidden_flag: HiddenFlag,
    /// 排序
    #[sea_orm(indexed)]
    pub sort_index: i32,
}

#[derive(Debug, Clone, Copy, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::super::system::sys_user::Entity",
        from = "Column::CreatorId",
        to = "super::super::system::sys_user::Column::Id"
    )]
    CreatorId,
    #[sea_orm(
        belongs_to = "super::super::system::sys_user::Entity",
        from = "Column::UpdaterId",
        to = "super::super::system::sys_user::Column::Id"
    )]
    UpdaterId,

    #[sea_orm(belongs_to = "Entity", from = "Column::ParentId", to = "Column::Id")]
    ParentId,
}

pub struct ParentReferencingLink;
impl Linked for ParentReferencingLink {
    type FromEntity = Entity;
    type ToEntity = Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::ParentId.def()]
    }
}

impl ActiveModelBehavior for ActiveModel {}
