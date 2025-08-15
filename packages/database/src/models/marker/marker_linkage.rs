use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

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
    pub group_id: Option<String>,
    /// 起始点点位 ID
    /// 会根据是否反向与 to_id 交换
    pub from_id: Option<i64>,
    /// 终止点点位 ID
    /// 会根据是否反向与 from_id 交换
    pub to_id: Option<i64>,
    /// 关联操作类型
    pub link_action: Option<String>,
    /// 是否反向
    pub link_reverse: Option<bool>,
    /// 路线
    /// 默认为空数组
    pub path: Option<serde_json::Value>,
    /// 额外数据
    pub extra: Option<serde_json::Value>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
