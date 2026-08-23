use anyhow::Result;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractManager;

/// 按标签名更新图标绑定（前端兼容路由）
/// POST /tag/{tagName}/{iconId}
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractManager(auth): ExtractManager,
    Path((tag_name, icon_id)): Path<(String, i64)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::tag::do_update_by_name(auth, tag_name, icon_id).await {
        Ok(resp) => Ok((StatusCode::OK, axum::Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
