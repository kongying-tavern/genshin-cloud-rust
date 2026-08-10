use serde::{Deserialize, Serialize};

/// 打点审核筛选请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PunctuateAuditFilterRequest {
    /// 地区 ID 列表
    pub area_id_list: Option<Vec<i64>>,
    /// 提交者 ID 列表
    pub author_list: Option<Vec<i64>>,
    /// 物品 ID 列表
    pub item_id_list: Option<Vec<i64>>,
    /// 类型 ID 列表
    pub type_id_list: Option<Vec<i64>>,
}
