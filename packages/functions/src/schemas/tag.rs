use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct DTO {
    // 乐观锁，修改次数
    pub version: Option<i64>,
    // 标签名
    pub tag: Option<String>,
    // 标签类型 ID 列表
    pub typeIdList: Option<Vec<i64>>,
    // 图标 ID
    pub iconId: Option<i64>,
    // 图标 URL
    pub url: Option<String>,
}
