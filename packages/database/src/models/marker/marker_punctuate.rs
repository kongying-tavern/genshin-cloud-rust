use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use _utils::types::{HiddenFlag, MarkerPunctuateMethodType, MarkerPunctuateStatus};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "marker_punctuate", schema_name = "genshin_map")]
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

    /// 点位提交 ID
    pub punctuate_id: i64,
    /// 原有点位 ID
    /// 该列的数值仅当操作为修改或删除时才有意义
    #[sea_orm(indexed)]
    pub original_marker_id: Option<i64>,
    /// 点位名称
    pub marker_title: Option<String>,
    /// 点位物品列表
    pub item_list: serde_json::Value,
    /// 点位坐标
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
    /// 点位提交者 ID
    pub author: i64,
    /// 状态
    pub status: MarkerPunctuateStatus,
    /// 审核备注
    pub audit_remark: Option<String>,
    /// 操作类型
    pub method_type: MarkerPunctuateMethodType,
    /// 点位刷新时间
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

    #[sea_orm(
        belongs_to = "super::super::system::sys_user::Entity",
        from = "Column::Author",
        to = "super::super::system::sys_user::Column::Id"
    )]
    Author,
}

impl ActiveModelBehavior for ActiveModel {}
