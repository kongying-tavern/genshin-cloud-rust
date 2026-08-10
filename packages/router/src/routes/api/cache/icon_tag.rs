use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 删除标签缓存
/// DELETE /cache/icon_tag
/// body 为要清除的 tag 列表（`string[]`，list 为空则删除所有标签缓存）
#[tracing::instrument(skip(auth))]
pub async fn delete_icon_tag_cache(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(tags): Json<Vec<String>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::cache::do_delete_icon_tag_cache(auth, tags).await {
        Ok(resp) => Ok((StatusCode::OK, axum::Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
