use serde::{Deserialize, Serialize};

use crate::models::Pagination;

/// 图标基础请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconBaseRequest {
    /// 图标名称
    pub name: String,
    /// 图标 URL
    pub url: String,
}

/// 新增图标请求
pub type IconAddRequest = IconBaseRequest;

/// 更新图标请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconUpdateRequest {
    /// 图标 ID
    pub id: i64,
    /// 乐观锁版本号
    pub version: i64,
    /// 基础图标信息
    #[serde(flatten)]
    pub base: IconBaseRequest,
}

/// 图标列表查询请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconListRequest {
    /// 上传者
    pub creator: Option<i64>,
    /// 图标 ID 列表（旧字段，兼容保留）
    pub icon_list: Option<Vec<i64>>,
    /// 图标 ID 列表（前端契约 iconIdList）
    pub icon_id_list: Option<Vec<i64>>,
    /// 图标分类 ID 列表（前端契约 typeIdList，按 icon_type_link 过滤）
    pub type_id_list: Option<Vec<i64>>,
    /// 图标名
    pub name: Option<String>,

    #[serde(flatten)]
    pub page: Pagination,
}

/// 图标返回值（用于 API 响应）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconVO {
    pub id: i64,
    /// 乐观锁版本号（前端编辑时随行提交，用于乐观锁校验）
    pub version: i64,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconListResponse {
    pub total: i64,
    #[serde(rename = "record")]
    pub items: Vec<IconVO>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconSingleResponse {
    pub item: IconVO,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconAddResponse {
    pub id: i64,
}
