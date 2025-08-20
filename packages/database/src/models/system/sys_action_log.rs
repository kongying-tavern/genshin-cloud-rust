use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use _utils::types::{SysActionLogExtra, SystemActionLogAction};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "sys_action_log", schema_name = "genshin_map")]
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

    /// 用户 ID
    pub user_id: Option<i64>,
    /// IPv4
    pub ipv4: Option<String>,
    /// 设备编码
    pub device_id: String,
    /// 操作名
    pub action: SystemActionLogAction,
    /// 是否发生错误
    pub is_error: bool,
    /// 附加信息
    #[sea_orm(column_type = "Json")]
    pub extra_data: SysActionLogExtra,
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
        belongs_to = "super::super::system::sys_user::Entity",
        from = "Column::UserId",
        to = "super::super::system::sys_user::Column::Id"
    )]
    UserId,
}

impl ActiveModelBehavior for ActiveModel {}
