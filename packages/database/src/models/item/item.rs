use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "item", schema_name = "genshin_map")]
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

    /// 物品名称
    pub name: String,
    /// 地区 ID
    /// 须确保是末端地区
    pub area_id: i64,
    /// 默认刷新时间
    /// 单位为毫秒
    pub default_refresh_time: i64,
    /// 默认描述模板
    /// 用于提交新物品点位时的描述模板
    pub default_content: Option<String>,
    /// 默认数量
    pub default_count: i32,
    /// 图标标签
    pub icon_tag: String,
    /// 图标样式类型
    pub icon_style_type: i32,
    /// 权限屏蔽标记
    /// 0: 可见, 1: 隐藏, 2: 内鬼, 3: 彩蛋
    pub hidden_flag: i32,
    /// 物品排序
    pub sort_index: i32,
    /// 特殊物品标记
    /// 低位第一位: 前台是否显示
    pub special_flag: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
