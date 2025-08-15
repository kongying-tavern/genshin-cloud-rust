use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

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
    pub t_id: i64,
    /// 操作数据类型
    /// 1: 地区, 2: 图标, 3: 物品, 4: 点位, 5: 标签
    #[sea_orm(column_name = "type")]
    pub history_type: Option<String>,
    /// IPv4
    pub ipv4: Option<String>,
    /// 修改类型
    /// 0: 未知, 1: 新增, 2: 修改, 3: 删除
    pub edit_type: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
