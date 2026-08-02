use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use _utils::{impl_safe_operation, types::MarkerLinkageLinkAction};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "marker_linkage", schema_name = "genshin_map")]
/// 可空性说明：旧库中 group_id / from_id / to_id / link_action / link_reverse
/// 均为「可空 + DEFAULT 兜底」（'' / 0 / false）且无任何 NULL 数据实例（461 MB
/// 备份扫描验证），因此实体保持非 Option（我们写入必填、读取安全）。
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
    pub group_id: String,
    /// 起始点点位 ID
    /// 会根据是否反向与 to_id 交换
    pub from_id: i64,
    /// 终止点点位 ID
    /// 会根据是否反向与 from_id 交换
    pub to_id: i64,
    /// 关联操作类型
    pub link_action: MarkerLinkageLinkAction,
    /// 是否反向
    pub link_reverse: bool,
    /// 路线
    /// 默认为空数组。类型保持宽松（`Option<Json>`）：当前数据库样本中该列
    /// 均为空数组，缺少真实数据案例，进一步的结构化（如固定字段的路线
    /// 类型）待真实数据验证后再收紧。
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

impl_safe_operation! {
    active_model_ty: ActiveModel,
    updated_at_column_name: update_time,
    updated_at_column_init_expr: chrono::Utc::now().naive_utc(),
    del_flag_column: Column::DelFlag
}
