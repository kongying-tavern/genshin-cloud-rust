use anyhow::Result;
use axum::{extract::Json, http::StatusCode, response::IntoResponse};
use _utils::models::notice::NoticeAddRequest;
use crate::middlewares::ExtractAuthInfo;

/// 新增公告
#[tracing::instrument(skip_all)]
pub async fn add_notice(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(request): Json<NoticeAddRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现公告新增逻辑
    Ok(())
}
