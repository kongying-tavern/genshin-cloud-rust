use serde::{Deserialize, Serialize};

use crate::{models::Pagination, types::HiddenFlag};

/// 标签基础请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagBaseRequest {
    /// 标签名
    pub tag: String,
    /// 图标 ID
    pub icon_id: i64,
}

/// 新增标签请求
pub type TagAddRequest = TagBaseRequest;

/// 更新标签请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagUpdateRequest {
    /// 标签 ID
    pub id: i64,
    /// 乐观锁版本号
    pub version: i64,
    /// 基础标签信息
    #[serde(flatten)]
    pub base: TagBaseRequest,
}

/// 更新标签类型关联请求（特殊接口用）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagUpdateTypeRequest {
    /// 标签名
    pub tag: String,
    /// 标签类型 ID 列表
    pub type_id_list: Vec<i64>,
}

/// 标签列表查询请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagListRequest {
    /// 标签名（模糊搜索）
    pub tag: Option<String>,
    /// 图标 ID
    pub icon_id: Option<i64>,

    #[serde(flatten)]
    pub page: Pagination,
}

/// 标签返回值 VO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagVO {
    pub id: i64,
    pub tag: String,
    pub icon_id: i64,
    pub hidden_flag: HiddenFlag,
    pub sort_index: i32,
}

/// 标签列表响应
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagListResponse {
    pub total: i64,
    pub list: Vec<TagVO>,
}

/// 标签新增响应
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagAddResponse {
    pub id: i64,
}
