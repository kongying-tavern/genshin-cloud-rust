use serde::{Deserialize, Serialize};

/// 点位物品关联请求模型
/// 用于创建点位与物品之间的关联关系
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerItemLinkRequest {
    /// 第一个关联 ID
    pub id1: i64,
    /// 第二个关联 ID
    pub id2: i64,
    /// 数量
    pub count: i32,
}

/// 带数量的关联请求模型
/// 用于需要包含数量信息的关联关系
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CountedLinkRequest {
    /// 第一个关联 ID
    pub id1: i64,
    /// 第二个关联 ID
    pub id2: i64,
    /// 数量
    pub count: i32,
}

/// 用于更新操作的关联请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLinkRequest {
    /// 关联记录 ID
    pub id: i64,
    /// 乐观锁版本号
    pub version: i64,
    /// 基础关联信息
    #[serde(flatten)]
    pub link: CountedLinkRequest,
}

/// 用于删除操作的简单关联请求模型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteLinkRequest {
    /// 第一个关联 ID
    pub id1: i64,
    /// 第二个关联 ID
    pub id2: i64,
}

/// 点位物品关联添加请求模型
pub type MarkerItemLinkAddRequest = CountedLinkRequest;

/// 点位物品关联更新请求模型
pub type MarkerItemLinkUpdateRequest = UpdateLinkRequest;

/// 点位物品关联删除请求模型
pub type MarkerItemLinkDeleteRequest = DeleteLinkRequest;
