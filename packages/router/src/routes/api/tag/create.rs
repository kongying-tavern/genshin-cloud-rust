use anyhow::Result;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractManager;

/// 新增标签（前端兼容路由，仅传标签名）
/// PUT /tag/{tagName}
#[tracing::instrument(skip(auth))]
pub async fn create(
    ExtractManager(auth): ExtractManager,
    Path(tag_name): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::tag::do_create_by_name(auth, tag_name).await {
        Ok(resp) => Ok((StatusCode::OK, axum::Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
