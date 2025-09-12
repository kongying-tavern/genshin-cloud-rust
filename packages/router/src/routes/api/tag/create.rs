use anyhow::Result;

use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};

use crate::middlewares::ExtractAuthInfo;

/// 创建标签
/// 只创建一个空标签
/// PUT /tag/{tagName}
#[tracing::instrument(skip(auth))]
pub async fn create(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(tag_name): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::tag::do_create(auth, tag_name).await {
        Ok(id) => Ok((StatusCode::OK, Json(serde_json::json!({"id": id})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
