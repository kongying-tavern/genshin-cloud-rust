use serde::{Deserialize, Serialize};

/// 评分数据生成请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreGenerateRequest {
    pub end_time: f64,
    pub scope: String,
    pub span: String,
    pub start_time: f64,
}

/// 评分数据获取请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreDataRequest {
    pub end_time: f64,
    pub scope: String,
    pub span: String,
    pub start_time: f64,
}
