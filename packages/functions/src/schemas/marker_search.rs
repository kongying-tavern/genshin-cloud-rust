use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct Schema {
    // 地区 ID 列表
    pub areaIdList: Option<Vec<i64>>,
    // 物品 ID 列表
    pub itemIdList: Option<Vec<i64>>,
    // 类型 ID 列表
    pub typeIdList: Option<Vec<i64>>,
    // 是否获取测试点位，默认不获取
    pub getBeta: Option<bool>,
    // 数据等级范围
    pub hiddenFlagList: Option<Vec<i32>>,
}
