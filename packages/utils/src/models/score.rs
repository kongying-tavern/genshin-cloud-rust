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

/// 单个评分样本
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreSample {
    pub time: f64,
    pub score: f64,
}

/// 评分生成/返回结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreResponse {
    pub samples: Vec<ScoreSample>,
    pub average: f64,
}
