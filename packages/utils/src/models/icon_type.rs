use serde::{Deserialize, Serialize};

/// 图标类型基础请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconTypeBaseRequest {
    /// 分类名
    pub name: String,
    /// 父级分类 ID（-1 为根分类）
    pub parent_id: i64,
    /// 是否为末端类型
    pub is_final: bool,
}

/// 新增图标类型请求
pub type IconTypeAddRequest = IconTypeBaseRequest;

/// 更新图标类型请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconTypeUpdateRequest {
    /// 图标类型ID
    pub id: u64,
    /// 乐观锁版本号
    pub version: u64,
    /// 基础图标类型信息
    #[serde(flatten)]
    pub base: IconTypeBaseRequest,
}
