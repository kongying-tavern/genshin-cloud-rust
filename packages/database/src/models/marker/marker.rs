use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use _utils::{impl_safe_operation, types::HiddenFlag};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "marker", schema_name = "genshin_map")]
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

    /// 点位签戳
    /// 用于兼容旧点位 ID
    #[deprecated = "仅用于兼容旧数据，现已不再使用"]
    pub marker_stamp: Option<String>,
    /// 点位名称
    pub marker_title: Option<String>,
    /// 点位坐标
    /// 形如 "{x},{y}" 的格式，其中 x 与 y 均为浮点数文本
    pub position: String,
    /// 点位说明
    pub content: String,
    /// 点位图片
    pub picture: Option<String>,
    /// 点位初始标记者
    pub marker_creator_id: i64,
    /// 点位图片上传者
    pub picture_creator_id: Option<i64>,
    /// 点位视频
    pub video_path: Option<String>,
    /// 点位刷新时间
    /// 单位为毫秒
    pub refresh_time: i64,
    /// 权限屏蔽标记
    #[sea_orm(indexed)]
    pub hidden_flag: HiddenFlag,
    /// 额外特殊字段
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
}

impl_safe_operation! {
    active_model_ty: ActiveModel,
    updated_at_column_name: update_time,
    updated_at_column_init_expr: chrono::Utc::now().naive_utc(),
    del_flag_column: Column::DelFlag
}
