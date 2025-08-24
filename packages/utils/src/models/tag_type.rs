use serde::{Deserialize, Serialize};

use crate::models::Pagination;

/// 标签类型基础请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagTypeBaseRequest {
    /// 分类名称
    pub name: String,
    /// 父级分类 ID（-1 为根分类）
    pub parent_id: i64,
    /// 是否为末端类型
    pub is_final: bool,
}

/// 新增标签类型请求
pub type TagTypeAddRequest = TagTypeBaseRequest;

/// 更新标签类型请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagTypeUpdateRequest {
    /// 标签类型 ID
    pub id: u64,
    /// 乐观锁版本号
    pub version: u64,
    /// 基础标签类型信息
    #[serde(flatten)]
    pub base: TagTypeBaseRequest,
}

/// 标签类型列表查询请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagTypeListRequest {
    /// 分类名称（模糊搜索）
    pub name: Option<String>,
    /// 父级分类 ID
    pub parent_id: Option<i64>,
    /// 是否遍历子类型
    pub is_traverse: Option<bool>,

    #[serde(flatten)]
    pub page: Pagination,
}
