use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct Schema {
    // 乐观锁，修改次数
    pub version: Option<i64>,
    // 地区 ID 列表
    pub areaIdList: Option<Vec<i64>>,
    // 物品 ID 列表
    pub itemIdList: Option<Vec<i64>>,
    // 类型 ID 列表
    pub typeIdList: Option<Vec<i64>>,
    // 提交者 ID 列表
    pub authorList: Option<Vec<i64>>,
}
