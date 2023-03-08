use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct Schema {
    // 父级 ID
    pub parentId: Option<i64>,
    // 是否遍历子地区
    pub isTraverse: Option<bool>,
    // 数据等级，即 hidden_flag 范围
    pub hiddenFlagList: Option<Vec<i32>>,
}
