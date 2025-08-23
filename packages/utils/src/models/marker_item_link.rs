use serde::{Deserialize, Serialize};

/// 点位物品关联基础请求模型
/// 去除了数据库模型中的ID、乐观锁、创建者/更新者信息和时间戳字段
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerItemLinkRequest {
    /// 物品 ID
    pub item_id: u64,
    /// 点位 ID
    pub marker_id: u64,
    /// 物品于该点位数量
    pub count: i32,
}

/// 点位物品关联添加请求模型
pub type MarkerItemLinkAddRequest = MarkerItemLinkRequest;

/// 点位物品关联更新请求模型
/// 使用 flatten 展开基础字段，避免重复定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerItemLinkUpdateRequest {
    /// 关联ID
    pub id: u64,
    /// 乐观锁版本号
    pub version: u64,
    /// 基础关联信息
    #[serde(flatten)]
    pub link: MarkerItemLinkRequest,
}

/// 点位物品关联删除请求模型
/// 用于删除特定的关联关系
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkerItemLinkDeleteRequest {
    /// 物品 ID
    pub item_id: u64,
    /// 点位 ID
    pub marker_id: u64,
}
