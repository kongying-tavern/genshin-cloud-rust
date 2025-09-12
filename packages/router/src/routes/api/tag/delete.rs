use anyhow::Result;

use axum::extract::Json;
use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 删除标签
/// 需要确保已经没有条目在使用这个标签，否则会删除失败
/// DELETE /tag/{tagName}
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(tag_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::tag::do_delete(auth, tag_id).await {
        Ok(_) => Ok((StatusCode::OK, Json(serde_json::json!({})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
