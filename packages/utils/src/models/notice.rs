use serde::{Deserialize, Serialize};

use crate::models::wrapper::Pagination;

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

/// 公告列表查询请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoticeListRequest {
    /// 频道
    pub channels: Option<Vec<NoticeChannel>>,
    /// 获取有效数据，true: 仅获取有效数据 false: 仅获取无效数据
    pub get_valid: Option<bool>,
    /// 排序条件
    pub sort: Option<Vec<NoticeSort>>,
    /// 标题
    pub title: Option<String>,
    /// 转换器名称
    pub transformer: Option<NoticeTransformer>,
    /// 分页参数
    #[serde(flatten)]
    pub page: Pagination,
}

/// 公告添加请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoticeAddRequest {
    /// 频道名
    pub channel: Vec<NoticeChannel>,
    /// 内容
    pub content: String,
    /// 排序
    pub sort_index: i64,
    /// 标题
    pub title: String,
    /// 有效期结束时间
    pub valid_time_end: Option<f64>,
    /// 有效期开始时间
    pub valid_time_start: Option<f64>,
}

/// 公告更新请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoticeUpdateRequest {
    /// 频道名
    pub channel: Vec<NoticeChannel>,
    /// 内容
    pub content: String,
    /// 公告 ID
    pub id: i64,
    /// 排序
    pub sort_index: i64,
    /// 标题
    pub title: String,
    /// 有效期结束时间
    pub valid_time_end: Option<f64>,
    /// 有效期开始时间
    pub valid_time_start: Option<f64>,
}
