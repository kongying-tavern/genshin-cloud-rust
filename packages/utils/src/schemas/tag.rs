use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct Schema {
    // 标签名
    pub tag: Option<String>,
    // 标签类型 ID 列表
    pub typeIdList: Option<Vec<i64>>,
    // 图标 ID
    pub iconId: Option<i64>,
    // 图标 URL
    pub url: Option<String>,
}
