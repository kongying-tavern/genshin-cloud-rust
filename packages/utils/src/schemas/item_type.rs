use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct Schema {
    // 类型 ID
    pub typeId: Option<i64>,
    // 图标标签
    pub iconTag: Option<String>,
    // 类型名
    pub name: Option<String>,
    // 类型补充说明
    pub content: Option<String>,
    // 父级类型 ID，没有父级类型时为 -1
    pub parentId: Option<i64>,
    // 是否为末端类型
    pub isFinal: Option<bool>,
    // 隐藏标志
    pub hiddenFlag: Option<i32>,
    // 物品类型排序
    pub sortIndex: Option<i32>,
}
