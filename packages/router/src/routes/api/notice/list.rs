use anyhow::Result;
use axum::{extract::Json, http::StatusCode, response::IntoResponse};
use _utils::models::notice::NoticeListRequest;
use crate::middlewares::ExtractAuthInfo;

/// 获取公告列表
#[tracing::instrument(skip_all)]
pub async fn get_notice_list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(request): Json<NoticeListRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现公告列表获取逻辑
    Ok(())
}
