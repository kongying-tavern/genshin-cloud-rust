use anyhow::Result;
use axum::{extract::Json, http::StatusCode, response::IntoResponse};
use _utils::models::notice::NoticeUpdateRequest;
use crate::middlewares::ExtractAuthInfo;

/// 更新公告
#[tracing::instrument(skip_all)]
pub async fn update_notice(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(request): Json<NoticeUpdateRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现公告更新逻辑
    Ok(())
}
