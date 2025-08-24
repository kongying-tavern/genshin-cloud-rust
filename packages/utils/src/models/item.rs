use serde::{Deserialize, Serialize};

use crate::{
    models::wrapper::Pagination,
    types::{HiddenFlag, IconStyleType},
};

/// 物品基础请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemRequest {
    /// 物品名称
    pub name: String,
    /// 地区 ID
    /// 须确保是末端地区
    pub area_id: u64,
    /// 默认刷新时间
    /// 单位为毫秒
    pub default_refresh_time: u64,
    /// 默认描述模板
    /// 用于提交新物品点位时的描述模板
    pub default_content: Option<String>,
    /// 默认数量
    pub default_count: i32,
    /// 图标标签
    pub icon_tag: String,
    /// 图标样式类型
    pub icon_style_type: IconStyleType,
    /// 权限屏蔽标记
    pub hidden_flag: HiddenFlag,
    /// 物品排序
    pub sort_index: i32,
    /// 特殊物品标记
    /// 低位第一位: 前台是否显示
    pub special_flag: Option<i32>,
}

/// 物品添加请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemAddRequest {
    /// 地区 ID
    pub area_id: i64,
    /// 默认描述模板
    pub default_content: String,
    /// 默认点位计数
    pub default_count: i64,
    /// 默认刷新时间
    pub default_refresh_time: Option<i64>,
    /// 权限屏蔽标记
    pub hidden_flag: HiddenFlag,
    /// 物品显示类型
    pub icon_style_type: IconStyleType,
    /// 图标标签
    pub icon_tag: String,
    /// 物品名称
    pub name: String,
    /// 排序
    pub sort_index: Option<i64>,
    /// 特殊物品标记，二进制表示；低位第一位：是否为显示物品
    pub special_flag: i64,
    /// 物品类型 ID 列表
    pub type_id_list: Vec<i64>,
}

/// 物品更新数据结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemUpdateData {
    /// 地区 ID
    pub area_id: i64,
    /// 默认描述模板
    pub default_content: String,
    /// 默认点位计数
    pub default_count: i64,
    /// 默认刷新时间
    pub default_refresh_time: Option<i64>,
    /// 权限屏蔽标记
    pub hidden_flag: HiddenFlag,
    /// 物品显示类型
    pub icon_style_type: IconStyleType,
    /// 图标标签
    pub icon_tag: String,
    /// 物品 ID
    pub id: i64,
    /// 物品名称
    pub name: String,
    /// 排序
    pub sort_index: Option<i64>,
    /// 特殊物品标记，二进制表示；低位第一位：是否为显示物品
    pub special_flag: i64,
    /// 物品类型 ID 列表
    pub type_id_list: Vec<i64>,
}

/// 物品列表查询排序枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ItemSort {
    #[serde(rename = "sortIndex-")]
    SortIndexDesc,
    #[serde(rename = "sortIndex+")]
    SortIndexAsc,
}

/// 物品筛选请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemFilterRequest {
    /// 末端地区 ID 列表
    pub area_id_list: Option<Vec<i64>>,
    /// 物品名，模糊查询
    pub name: Option<String>,
    /// 排序
    pub sort: Option<Vec<ItemSort>>,
    /// 特殊标记
    pub special_flag: Option<f64>,
    /// 末端物品类型 ID 列表
    pub type_id_list: Option<Vec<i64>>,
    /// 分页参数
    #[serde(flatten)]
    pub page: Pagination,
}

/// 物品更新请求模型
/// 使用 flatten 展开基础字段，避免重复定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemUpdateRequest {
    /// 物品 ID
    pub id: u64,
    /// 乐观锁版本号
    pub version: u64,
    /// 基础物品信息
    #[serde(flatten)]
    pub item: ItemRequest,
}

/// 物品列表查询请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemListRequest {
    /// 物品名称（模糊搜索）
    pub name: Option<String>,
    /// 地区 ID
    pub area_id: Option<u64>,
    /// 权限屏蔽标记
    pub hidden_flag: Option<HiddenFlag>,

    #[serde(flatten)]
    pub page: Pagination,
}
