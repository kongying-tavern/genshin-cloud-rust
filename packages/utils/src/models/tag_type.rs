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
    #[serde(default)]
    pub is_final: bool,
}

/// 新增标签类型请求
pub type TagTypeAddRequest = TagTypeBaseRequest;

/// 更新标签类型请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagTypeUpdateRequest {
    /// 标签类型 ID
    pub id: i64,
    /// 乐观锁版本号
    pub version: i64,
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
    /// 分类 ID 列表（前端契约 typeIdList；-1 表示根/全部，仅对正数 ID 过滤）
    pub type_id_list: Option<Vec<i64>>,
    /// 是否遍历子类型
    pub is_traverse: Option<bool>,

    #[serde(flatten)]
    pub page: Pagination,
}

/// 标签类型返回值 VO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagTypeVO {
    /// 乐观锁版本号（前端编辑时随行提交，用于乐观锁校验）
    pub version: i64,
    pub id: i64,
    pub create_time: f64,
    pub update_time: Option<f64>,
    pub creator_id: Option<i64>,
    pub updater_id: Option<i64>,
    pub name: String,
    pub parent_id: i64,
    pub is_final: bool,
}

/// 标签类型列表响应
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagTypeListResponse {
    pub total: i64,
    #[serde(rename = "record")]
    pub list: Vec<TagTypeVO>,
}

/// 标签类型新增响应
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagTypeAddResponse {
    pub id: i64,
}
