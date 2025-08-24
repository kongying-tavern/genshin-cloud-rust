use serde::{Deserialize, Serialize};

use crate::{
    models::wrapper::Pagination,
    types::{HistoryEditType, HistoryOperationType},
};

/// 历史记录排序枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HistorySort {
    #[serde(rename = "updateTime+")]
    UpdateTimeAsc,
    #[serde(rename = "updateTime-")]
    UpdateTimeDesc,
}

/// 历史记录分页查询请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryListRequest {
    /// 创建时间结束时间
    pub create_time_end: Option<f64>,
    /// 创建时间开始时间
    pub create_time_start: Option<f64>,
    /// 创建人 ID
    pub creator_id: Option<String>,
    /// 编辑类型
    pub edit_type: Option<HistoryEditType>,
    /// 类型下的 ID (配合记录类型使用)
    pub id: Option<Vec<i64>>,
    /// 排序
    pub sort: Option<Vec<HistorySort>>,
    /// 记录类型
    #[serde(rename = "type")]
    pub request_type: Option<HistoryOperationType>,
    /// 分页参数
    #[serde(flatten)]
    pub page: Pagination,
}
