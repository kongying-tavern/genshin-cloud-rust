use serde::{Deserialize, Serialize};

/// 公告频道枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NoticeChannel {
    Application,
    #[serde(rename = "CLIENT_APP")]
    ClientApp,
    #[serde(rename = "CLIENT_PC")]
    ClientPc,
    Common,
    Dadian,
    Dashboard,
    Tianli,
    Web,
}

/// 公告排序枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NoticeSort {
    #[serde(rename = "isValid+")]
    IsValidAsc,
    #[serde(rename = "isValid-")]
    IsValidDesc,
    #[serde(rename = "sortIndex+")]
    SortIndexAsc,
    #[serde(rename = "sortIndex-")]
    SortIndexDesc,
    #[serde(rename = "title+")]
    TitleAsc,
    #[serde(rename = "title-")]
    TitleDesc,
    #[serde(rename = "updateTime+")]
    UpdateTimeAsc,
    #[serde(rename = "updateTime-")]
    UpdateTimeDesc,
    #[serde(rename = "validTimeEnd+")]
    ValidTimeEndAsc,
    #[serde(rename = "validTimeEnd-")]
    ValidTimeEndDesc,
    #[serde(rename = "validTimeStart+")]
    ValidTimeStartAsc,
    #[serde(rename = "validTimeStart-")]
    ValidTimeStartDesc,
    #[serde(rename = "validType+")]
    ValidTypeAsc,
    #[serde(rename = "validType-")]
    ValidTypeDesc,
}

/// 转换器名称枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NoticeTransformer {
    Unity,
}

use crate::models::wrapper::Pagination;

/// 公告添加请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoticeAddRequest {
    pub channel: Vec<NoticeChannel>,
    pub content: String,
    pub sort_index: i64,
    pub title: String,
    pub valid_time_end: Option<f64>,
    pub valid_time_start: Option<f64>,
}

/// 公告列表查询请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoticeListRequest {
    pub channels: Option<Vec<NoticeChannel>>,
    pub get_valid: Option<bool>,
    pub sort: Option<Vec<NoticeSort>>,
    pub title: Option<String>,
    pub transformer: Option<NoticeTransformer>,
    #[serde(flatten)]
    pub page: Pagination,
}

/// 公告更新请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoticeUpdateRequest {
    pub channel: Vec<NoticeChannel>,
    pub content: String,
    pub id: i64,
    pub sort_index: i64,
    pub title: String,
    pub valid_time_end: Option<f64>,
    pub valid_time_start: Option<f64>,
}
