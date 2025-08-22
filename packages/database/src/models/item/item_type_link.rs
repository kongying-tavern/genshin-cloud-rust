use _utils::impl_safe_operation;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "item_type_link", schema_name = "genshin_map")]
pub struct Model {
    /// 乐观锁
    pub version: u64,
    /// ID
    #[sea_orm(primary_key)]
    pub id: u64,
    /// 创建时间
    pub create_time: DateTime,
    /// 更新时间
    pub update_time: Option<DateTime>,
    /// 创建人
    pub creator_id: Option<u64>,
    /// 更新人
    pub updater_id: Option<u64>,
    /// 逻辑删除
    pub del_flag: bool,

    /// 类型 ID
    /// 此处必须为末端类型
    pub type_id: u64,
    /// 物品 ID
    #[sea_orm(indexed)]
    pub item_id: u64,
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

    #[sea_orm(
        belongs_to = "super::item_type::Entity",
        from = "Column::TypeId",
        to = "super::item_type::Column::Id"
    )]
    TypeId,
    #[sea_orm(
        belongs_to = "super::item::Entity",
        from = "Column::ItemId",
        to = "super::item::Column::Id"
    )]
    ItemId,
}

impl_safe_operation! {
    active_model_ty: ActiveModel,
    updated_at_column_name: update_time,
    updated_at_column_init_expr: chrono::Utc::now().naive_utc(),
    del_flag_column: Column::DelFlag
}
