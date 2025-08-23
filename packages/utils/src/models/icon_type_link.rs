use serde::{Deserialize, Serialize};

/// 图标类型关联基础请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconTypeLinkRequest {
    /// 分类 ID
    pub type_id: u64,
    /// 图标 ID
    pub icon_id: u64,
}

/// 图标类型关联添加请求模型
pub type IconTypeLinkAddRequest = IconTypeLinkRequest;

/// 图标类型关联删除请求模型
/// 用于删除特定的关联关系
pub type IconTypeLinkDeleteRequest = IconTypeLinkRequest;
