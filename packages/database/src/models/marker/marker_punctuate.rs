use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

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
    /// 0: 暂存, 1: 审核中, 2: 不通过
    pub status: i32,
    /// 审核备注
    pub audit_remark: Option<String>,
    /// 操作类型
    /// 1: 新增, 2: 修改, 3: 删除
    pub method_type: i32,
    /// 点位刷新时间
    pub refresh_time: i64,
    /// 权限屏蔽标记
    /// 0: 可见, 1: 隐藏, 2: 内鬼, 3: 彩蛋
    pub hidden_flag: i32,
    /// 额外特殊字段
    pub extra: Option<serde_json::Value>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
