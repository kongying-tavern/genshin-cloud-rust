use anyhow::Result;

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use axum::{http::StatusCode, response::IntoResponse};

/// 删除标签缓存
/// DELETE /cache/icon_tag
/// body 为要清除的 tag 列表（`string[]`，list 为空则删除所有标签缓存）
#[tracing::instrument(skip(auth))]
pub async fn delete_icon_tag_cache(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(tags): AppJson<Vec<String>>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::cache::do_delete_icon_tag_cache(auth, tags).await {
        Ok(resp) => Ok((StatusCode::OK, axum::Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
