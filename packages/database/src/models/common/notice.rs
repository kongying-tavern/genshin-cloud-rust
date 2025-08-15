use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "notice", schema_name = "genshin_map")]
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

    /// 频道
    /// 值为一个包含若干字符串的数组，默认为一个空数组
    pub channel: serde_json::Value,
    /// 标题
    pub title: String,
    /// 内容
    pub content: Option<String>,
    /// 有效期开始时间
    pub valid_time_start: Option<DateTime>,
    /// 有效期结束时间
    pub valid_time_end: Option<DateTime>,
    /// 排序
    pub sort_index: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
