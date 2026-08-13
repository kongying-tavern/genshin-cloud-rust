use anyhow::Result;

use crate::middlewares::{ApiError, ExtractAuthInfo};
use axum::{extract::Path, http::StatusCode, response::IntoResponse};

/// 删除图标
/// DELETE /icon/delete/{iconId}
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(icon_id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::icon::do_delete(auth, icon_id).await {
        Ok(resp) => Ok((StatusCode::OK, axum::Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
