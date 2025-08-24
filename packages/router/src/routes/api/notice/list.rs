use anyhow::Result;
use serde::{Deserialize, Serialize};

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::{
    notice::{NoticeChannel, NoticeSort, NoticeTransformer},
    Pagination,
};

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

/// 获取公告列表
#[tracing::instrument(skip_all)]
pub async fn get_notice_list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(request): Json<NoticeListRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现公告列表获取逻辑
    Ok(())
}
