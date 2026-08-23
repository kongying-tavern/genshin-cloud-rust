use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractManager;

/// 软删除标签
/// DELETE /tag/delete/{tagId}
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractManager(auth): ExtractManager,
    Path(tag_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::tag::do_delete(auth, tag_id).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
