use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use _utils::{impl_safe_operation, types::ScopeStatType};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "score_stat", schema_name = "genshin_map")]
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

    /// 统计范围
    #[sea_orm(indexed)]
    pub scope: String,
    /// 统计颗粒度
    #[sea_orm(indexed)]
    pub span: String,
    /// 统计开始时间
    #[sea_orm(indexed)]
    pub span_start_time: DateTime,
    /// 统计终止时间
    #[sea_orm(indexed)]
    pub span_end_time: DateTime,
    /// 用户 ID
    #[sea_orm(indexed)]
    pub user_id: Option<u64>,
    /// 统计内容 JSON
    pub content: ScopeStatType,
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
}

impl_safe_operation! {
    active_model_ty: ActiveModel,
    updated_at_column_name: update_time,
    updated_at_column_init_expr: chrono::Utc::now().naive_utc(),
    del_flag_column: Column::DelFlag
}
