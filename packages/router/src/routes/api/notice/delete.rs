use anyhow::Result;

use crate::middlewares::{ApiError, ExtractAuthInfo};
use axum::extract::Json;
use axum::{extract::Path, response::IntoResponse};

/// 删除公告
#[tracing::instrument(skip(auth))]
pub async fn delete_notice(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(notice_id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::notice::do_delete_notice(auth, notice_id).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
