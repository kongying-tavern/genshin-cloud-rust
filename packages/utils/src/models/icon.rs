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
    /// 图标ID
    pub id: u64,
    /// 乐观锁版本号
    pub version: u64,
    /// 基础图标信息
    #[serde(flatten)]
    pub base: IconBaseRequest,
}

/// 图标列表查询请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconListRequest {
    /// 上传者
    pub creator: Option<u64>,
    /// 图标 ID 列表
    pub icon_list: Option<Vec<u64>>,
    /// 图标名
    pub name: Option<String>,

    #[serde(flatten)]
    pub page: Pagination,
}
