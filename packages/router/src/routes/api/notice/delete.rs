use anyhow::Result;

use axum::extract::Json;
use axum::{extract::Path, response::IntoResponse};

use crate::middlewares::ExtractAdmin;

/// 删除公告
#[tracing::instrument(skip(auth))]
pub async fn delete_notice(
    ExtractAdmin(auth): ExtractAdmin,
    Path(notice_id): Path<i64>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::notice::do_delete_notice(auth, notice_id).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
