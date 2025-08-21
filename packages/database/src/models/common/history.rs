use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use _utils::{
    impl_safe_operation,
    types::{HistoryEditType, HistoryOperationType},
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "history", schema_name = "genshin_map")]
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

    /// 内容
    pub content: String,
    /// MD5
    pub md5: Option<String>,
    /// 原 ID
    #[sea_orm(indexed)]
    pub t_id: i64,
    /// 操作数据类型
    #[sea_orm(column_name = "type", indexed)]
    pub history_type: Option<HistoryOperationType>,
    /// IPv4
    pub ipv4: Option<String>,
    /// 修改类型
    pub edit_type: HistoryEditType,
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
