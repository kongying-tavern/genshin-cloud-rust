use serde::{Deserialize, Serialize};

/// 物品类型关联基础请求模型
/// 去除了数据库模型中的ID、乐观锁、创建者/更新者信息和时间戳字段
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemTypeLinkRequest {
    /// 类型 ID
    /// 此处必须为末端类型
    pub type_id: u64,
    /// 物品 ID
    pub item_id: u64,
}

/// 物品类型关联添加请求模型
pub type ItemTypeLinkAddRequest = ItemTypeLinkRequest;

/// 物品类型关联删除请求模型
/// 用于删除特定的关联关系
pub type ItemTypeLinkDeleteRequest = ItemTypeLinkRequest;
