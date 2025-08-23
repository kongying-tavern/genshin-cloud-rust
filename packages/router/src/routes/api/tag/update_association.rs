use anyhow::Result;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 修改标签关联
/// POST /tag/{tagName}/{iconId}
#[tracing::instrument(skip_all)]
pub async fn update_association(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path((tag_name, icon_id)): Path<(String, i64)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
