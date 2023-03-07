use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct DTO {
    // 统计 ID
    pub id: Option<i64>,
    // 统计范围
    pub scope: Option<String>,
    // 统计颗粒度
    pub span: Option<String>,
    // 统计起始时间
    pub spanStartTime: Option<NaiveDateTime>,
    // 统计结束时间
    pub spanEndTime: Option<NaiveDateTime>,
    // 用户 ID
    pub userId: Option<i64>,
    // 修改的字段 JSON
    pub content: Option<String>,
}
