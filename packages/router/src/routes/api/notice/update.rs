use anyhow::Result;

use axum::{extract::Json, response::IntoResponse};

use crate::middlewares::ExtractAdmin;
use _utils::models::notice::NoticeUpdateRequest;

/// 更新公告
#[tracing::instrument(skip(auth))]
pub async fn update_notice(
    ExtractAdmin(auth): ExtractAdmin,
    Json(request): Json<NoticeUpdateRequest>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::notice::do_update_notice(auth, request).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
