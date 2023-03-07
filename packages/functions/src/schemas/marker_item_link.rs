use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct DTO {
    // 物品 ID
    pub itemId: Option<i64>,
    // 点位物品数量
    pub count: Option<i32>,
    // 图标标签
    pub iconTag: Option<String>,
}
