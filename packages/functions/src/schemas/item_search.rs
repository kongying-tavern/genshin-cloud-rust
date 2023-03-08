use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct Schema {
    // 末端物品类型 ID 列表
    pub typeIdList: Option<Vec<i64>>,
    // 末端地区 ID 列表
    pub areaIdList: Option<Vec<i64>>,
    // 当前页，从 0 开始
    pub current: Option<i64>,
    // 每页大小，默认为 10
    pub size: Option<i64>,
    // 数据等级，即 hidden_flag 范围
    pub hiddenFlagList: Option<Vec<i32>>,
}
