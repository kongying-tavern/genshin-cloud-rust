use serde::{Deserialize, Serialize};

/// 评分数据操作请求
/// 用于生成和获取评分数据，两个操作使用相同的参数结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreRequest {
    /// 结束时间戳
    pub end_time: f64,
    /// 打点模块，可选值：punctuate - 打点
    pub scope: String,
    /// 统计颗粒度，可选值：day - 按天
    pub span: String,
    /// 开始时间戳
    pub start_time: f64,
}

/// 评分数据生成请求的类型别名
pub type ScoreGenerateRequest = ScoreRequest;

/// 评分数据获取请求的类型别名
pub type ScoreDataRequest = ScoreRequest;
