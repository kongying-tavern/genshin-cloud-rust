use anyhow::Result;
use serde::{Deserialize, Serialize};

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::notice::NoticeChannel;

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

/// 新增公告
#[tracing::instrument(skip(auth))]
pub async fn add_notice(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(request): Json<NoticeAddRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现公告新增逻辑
    Ok(())
}
