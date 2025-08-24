use anyhow::Result;
use serde::{Deserialize, Serialize};

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 评分数据获取请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreDataRequest {
    /// 结束时间戳
    pub end_time: f64,
    /// 打点模块，可选值：punctuate - 打点
    pub scope: String,
    /// 统计颗粒度，可选值：day - 按天
    pub span: String,
    /// 开始时间戳
    pub start_time: f64,
}

/// 获取评分数据
#[tracing::instrument(skip_all)]
pub async fn get_score_data(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(request): Json<ScoreDataRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现评分数据获取逻辑
    Ok(())
}
