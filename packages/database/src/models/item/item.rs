use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use _utils::{
    impl_safe_operation,
    types::{HiddenFlag, IconStyleType},
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "item", schema_name = "genshin_map")]
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

    /// 物品名称
    pub name: String,
    /// 地区 ID
    /// 须确保是末端地区
    #[sea_orm(indexed)]
    pub area_id: u64,
    /// 默认刷新时间
    /// 单位为毫秒
    #[sea_orm(default_value = 0)]
    pub default_refresh_time: u64,
    /// 默认描述模板
    /// 用于提交新物品点位时的描述模板
    pub default_content: Option<String>,
    /// 默认数量
    #[sea_orm(default_value = 1)]
    pub default_count: i32,
    /// 图标标签
    pub icon_tag: String,
    /// 图标样式类型
    pub icon_style_type: IconStyleType,
    /// 权限屏蔽标记
    #[sea_orm(indexed)]
    pub hidden_flag: HiddenFlag,
    /// 物品排序
    #[sea_orm(indexed)]
    pub sort_index: i32,
    /// 特殊物品标记
    /// 低位第一位: 前台是否显示
    pub special_flag: Option<i32>,
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
        belongs_to = "super::super::area::area::Entity",
        from = "Column::AreaId",
        to = "super::super::area::area::Column::Id"
    )]
    AreaId,
    #[sea_orm(
        belongs_to = "super::super::icon::icon::Entity",
        from = "Column::IconTag",
        to = "super::super::icon::icon::Column::Name"
    )]
    IconTag,
}

impl_safe_operation! {
    active_model_ty: ActiveModel,
    updated_at_column_name: update_time,
    updated_at_column_init_expr: chrono::Utc::now().naive_utc(),
    del_flag_column: Column::DelFlag
}
