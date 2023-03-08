use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct Schema {
    // 乐观锁，修改次数
    pub version: Option<i64>,
    // 图标 ID
    pub iconId: Option<i64>,
    // 图标名称
    pub name: Option<String>,
    // 图标类型 ID 列表
    pub iconTypeIds: Option<Vec<i64>>,
    // 图标 URL
    pub url: Option<String>,
    // 创建者 ID
    pub creator: Option<i64>,
}
