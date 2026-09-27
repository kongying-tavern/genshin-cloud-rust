use anyhow::Result;

use axum::{
    extract::{Json, State},
    response::IntoResponse,
};

use crate::middlewares::ExtractAdmin;
use _utils::models::notice::NoticeAddRequest;

/// 新增公告
#[tracing::instrument(skip(db, auth))]
pub async fn add_notice(
    State(db): State<crate::routes::SharedDb>,
    ExtractAdmin(auth): ExtractAdmin,
    Json(request): Json<NoticeAddRequest>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::notice::do_add_notice(db.as_ref(), auth, request).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
