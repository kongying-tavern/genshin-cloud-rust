use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "route", schema_name = "genshin_map")]
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

    /// 路线名称
    pub name: String,
    /// 路线描述
    pub content: Option<String>,
    /// 点位顺序数组
    pub marker_list: serde_json::Value,
    /// 权限屏蔽标记
    /// 0: 可见, 1: 隐藏, 2: 内鬼, 3: 彩蛋
    pub hidden_flag: i32,
    /// 视频地址
    pub video: Option<String>,
    /// 额外信息
    pub extra: serde_json::Value,
    /// 创建人昵称
    pub creator_nickname: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
