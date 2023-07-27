use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct Schema {
    // 地区 ID
    pub areaId: Option<i64>,
    // 地区名称
    pub name: Option<String>,
    // 地区代码
    pub code: Option<String>,
    // 地区描述
    pub content: Option<String>,
    // 图标标签
    pub iconTag: Option<String>,
    // 父级地区 ID，没有父级地区时为 -1
    pub parentId: Option<i64>,
    // 是否为末端地区
    pub isFinal: Option<bool>,
    // 隐藏标志
    pub hiddenFlag: Option<i32>,
    // 地区排序
    pub sortIndex: Option<i32>,
    // 特殊物品标记，低位第 1 位为 1 时则逻辑删除
    pub specialFlag: Option<i32>,
}

// TODO - 完善所有 Schema 的初始化标签
impl Default for Schema {
    fn default() -> Self {
        Self {
            areaId: Default::default(),
            name: Default::default(),
            code: Default::default(),
            content: Default::default(),
            iconTag: Default::default(),
            parentId: Default::default(),
            isFinal: Default::default(),
            hiddenFlag: Default::default(),
            sortIndex: Default::default(),
            specialFlag: Default::default(),
        }
    }
}
