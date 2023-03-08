use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct Schema {
    // 乐观锁，修改次数
    pub version: Option<i64>,
    // 物品 ID
    pub itemId: Option<String>,
    // 物品名称
    pub name: Option<String>,
    // 物品类型 ID 列表
    pub typeIdList: Option<Vec<i64>>,
    // 地区 ID，须确保是末端地区
    pub areaId: Option<i64>,
    // 提交新物品点位时的默认描述模板
    pub defaultContent: Option<String>,
    // 图标标签
    pub iconTag: Option<String>,
    // 图标样式类型
    pub iconStyleType: Option<i32>,
    // 隐藏标志
    pub hiddenFlag: Option<i32>,
    // 刷新时间，单位为毫秒
    pub defaultRefreshTime: Option<i64>,
    // 物品排序
    pub sortIndex: Option<i32>,
    // 默认物品数量
    pub defaultCount: Option<i32>,
    // 特殊物品标记，低位第 1 位为 1 时则逻辑删除
    pub specialFlag: Option<i32>,
    // 查询条件下物品总数
    pub count: Option<i32>,
}
