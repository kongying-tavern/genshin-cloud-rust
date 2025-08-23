use serde::{Deserialize, Serialize};

use crate::types::{HiddenFlag, IconStyleType};

/// 物品基础请求模型
/// 去除了数据库模型中的ID、乐观锁、创建者/更新者信息和时间戳字段
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
pub type ItemAddRequest = ItemRequest;

/// 物品更新请求模型
/// 使用 flatten 展开基础字段，避免重复定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemUpdateRequest {
    /// 物品ID
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
    /// 分页页码
    pub page: Option<u32>,
    /// 分页大小
    pub page_size: Option<u32>,
}
