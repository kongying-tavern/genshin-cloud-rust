use anyhow::Result;

use crate::middlewares::{ApiError, ExtractAuthInfo};
use axum::{extract::Path, http::StatusCode, response::IntoResponse};

/// 删除分类
/// 这个操作会递归删除，请在前端做二次确认
/// DELETE /icon_type/delete/{typeId}
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(type_id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::icon_type::do_delete(auth, type_id).await {
        Ok(resp) => Ok((StatusCode::OK, axum::Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
