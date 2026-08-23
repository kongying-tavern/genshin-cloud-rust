use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAdmin;
use _utils::models::notice::NoticeAddRequest;

/// 新增公告
#[tracing::instrument(skip(auth))]
pub async fn add_notice(
    ExtractAdmin(auth): ExtractAdmin,
    Json(request): Json<NoticeAddRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::notice::do_add_notice(auth, request).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
