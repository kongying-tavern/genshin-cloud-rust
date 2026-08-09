use serde::{Deserialize, Serialize};

/// 评分数据生成请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreGenerateRequest {
    #[serde(default)]
    pub end_time: f64,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub span: String,
    #[serde(default)]
    pub start_time: f64,
    /// 生成人 ID（前端可选字段）
    pub generator_id: Option<i64>,
}

/// 评分数据获取请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreDataRequest {
    #[serde(default)]
    pub end_time: f64,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub span: String,
    #[serde(default)]
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
