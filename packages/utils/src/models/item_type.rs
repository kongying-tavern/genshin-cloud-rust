use serde::{Deserialize, Serialize};

use crate::types::HiddenFlag;

/// 物品类型基础请求模型
/// 去除了数据库模型中的ID、乐观锁、创建者/更新者信息和时间戳字段
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemTypeRequest {
    /// 图标标签
    pub icon_tag: String,
    /// 类型名
    pub name: String,
    /// 类型补充说明
    pub content: Option<String>,
    /// 父级类型 ID
    /// 无父级则为 -1
    pub parent_id: i64,
    /// 是否为末端类型
    pub is_final: bool,
    /// 权限屏蔽标记
    pub hidden_flag: HiddenFlag,
    /// 排序
    pub sort_index: i32,
}

/// 物品类型添加请求模型
pub type ItemTypeAddRequest = ItemTypeRequest;

/// 物品类型更新请求模型
/// 使用 flatten 展开基础字段，避免重复定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemTypeUpdateRequest {
    /// 物品类型ID
    pub id: u64,
    /// 乐观锁版本号
    pub version: u64,
    /// 基础物品类型信息
    #[serde(flatten)]
    pub item_type: ItemTypeRequest,
}

/// 物品类型列表查询请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemTypeListRequest {
    /// 类型名（模糊搜索）
    pub name: Option<String>,
    /// 父级类型 ID
    pub parent_id: Option<i64>,
    /// 权限屏蔽标记
    pub hidden_flag: Option<HiddenFlag>,
    /// 是否遍历子类型
    pub is_traverse: Option<bool>,
}
