use serde::{Deserialize, Serialize};

/// 标签类型关联基础请求模型
/// 去除了数据库模型中的ID、乐观锁、创建者/更新者信息和时间戳字段
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagTypeLinkRequest {
    /// 分类 ID
    pub type_id: u64,
    /// 标签名称
    pub tag_name: String,
}

/// 标签类型关联添加请求模型
pub type TagTypeLinkAddRequest = TagTypeLinkRequest;

/// 标签类型关联删除请求模型
/// 用于删除特定的关联关系
pub type TagTypeLinkDeleteRequest = TagTypeLinkRequest;
