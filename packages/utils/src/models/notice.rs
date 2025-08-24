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
