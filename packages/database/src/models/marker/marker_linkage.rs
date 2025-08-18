use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use _utils::types::enums::MarkerLinkageLinkAction;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "marker_linkage", schema_name = "genshin_map")]
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

    /// 组 ID
    /// TODO: 更正实际数据库中的表关系，这个列其实不能为空
    pub group_id: String,
    /// 起始点点位 ID
    /// 会根据是否反向与 to_id 交换
    /// TODO: 更正实际数据库中的表关系，这个列其实不能为空
    pub from_id: i64,
    /// 终止点点位 ID
    /// 会根据是否反向与 from_id 交换
    /// TODO: 更正实际数据库中的表关系，这个列其实不能为空
    pub to_id: i64,
    /// 关联操作类型
    /// TODO: 更正实际数据库中的表关系，这个列其实不能为空
    pub link_action: MarkerLinkageLinkAction,
    /// 是否反向
    /// TODO: 更正实际数据库中的表关系，这个列其实不能为空
    pub link_reverse: bool,
    /// 路线
    /// 默认为空数组
    /// FIXME: 数据库里没有任何数据案例，全是空数组，这里的进一步类型限制就暂时做不了
    pub path: Option<serde_json::Value>,
    /// 额外数据
    pub extra: Option<serde_json::Value>,
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
        belongs_to = "super::marker::Entity",
        from = "Column::FromId",
        to = "super::marker::Column::Id"
    )]
    FromId,
    #[sea_orm(
        belongs_to = "super::marker::Entity",
        from = "Column::ToId",
        to = "super::marker::Column::Id"
    )]
    ToId,
}

impl ActiveModelBehavior for ActiveModel {}
