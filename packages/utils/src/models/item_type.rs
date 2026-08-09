use serde::{Deserialize, Serialize};

use crate::{models::wrapper::Pagination, types::HiddenFlag};

/// 物品类型对外 VO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemTypeVO {
    pub version: i64,
    pub id: i64,
    pub create_time: f64,
    pub update_time: Option<f64>,
    pub creator_id: Option<i64>,
    pub updater_id: Option<i64>,
    pub name: String,
    pub icon_tag: Option<String>,
    /// 图标 ID（远程 schema 列）
    pub icon_id: i64,
    pub content: Option<String>,
    pub parent_id: i64,
    pub is_final: bool,
    pub hidden_flag: HiddenFlag,
    pub sort_index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemTypeListResponse {
    pub total: i64,
    #[serde(rename = "record")]
    pub items: Vec<ItemTypeVO>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemTypeAllResponse(pub Vec<ItemTypeVO>);

/// 物品类型基础请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemTypeRequest {
    /// 图标标签
    #[serde(default)]
    pub icon_id: i64,
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemTypeAddRequest {
    /// 类型补充说明
    pub content: Option<String>,
    /// 权限屏蔽标记
    #[serde(default)]
    pub hidden_flag: HiddenFlag,
    /// 图标标签
    #[serde(default)]
    pub icon_id: i64,
    /// 图标标签名（前端以 iconTag 提交；iconId 为 0 时按此查 tag 表解析）
    #[serde(default)]
    pub icon_tag: Option<String>,
    /// 是否为末端地区
    #[serde(default)]
    pub is_final: bool,
    /// 类型名
    pub name: Option<String>,
    /// 父级 ID，无父级为 -1
    pub parent_id: i64,
    /// 排序
    pub sort_index: Option<i64>,
}

/// 物品类型更新请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemTypeUpdateData {
    /// 类型补充说明
    pub content: Option<String>,
    /// 权限屏蔽标记
    pub hidden_flag: HiddenFlag,
    /// 图标标签
    #[serde(default)]
    pub icon_id: i64,
    /// 图标标签名（前端以 iconTag 提交；iconId 为 0 时按此查 tag 表解析）
    #[serde(default)]
    pub icon_tag: Option<String>,
    /// 物品类型 ID
    pub id: i64,
    /// 是否为末端地区
    pub is_final: bool,
    /// 类型名
    pub name: Option<String>,
    /// 父级 ID，无父级为 -1
    pub parent_id: i64,
    /// 排序
    pub sort_index: Option<i64>,
}

/// 物品类型列表查询请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemTypeListRequest {
    /// 父级类型 ID 列表
    pub type_id_list: Option<Vec<i64>>,
    /// 分页参数
    #[serde(flatten)]
    pub page: Pagination,
}

/// 原有的物品类型更新请求模型
/// 使用 flatten 展开基础字段，避免重复定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemTypeUpdateRequest {
    /// 物品类型 ID
    pub id: i64,
    /// 乐观锁版本号
    pub version: i64,
    /// 基础物品类型信息
    #[serde(flatten)]
    pub item_type: ItemTypeRequest,
}

/// 原有的物品类型列表查询请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemTypeListLegacyRequest {
    /// 类型名（模糊搜索）
    pub name: Option<String>,
    /// 父级类型 ID
    pub parent_id: Option<i64>,
    /// 权限屏蔽标记
    pub hidden_flag: Option<HiddenFlag>,
    /// 是否遍历子类型
    pub is_traverse: Option<bool>,
}
