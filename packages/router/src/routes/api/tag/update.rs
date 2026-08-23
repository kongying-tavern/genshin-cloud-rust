use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractManager;
use _utils::models::TagUpdateRequest;

/// 更新标签
/// POST /tag/update
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractManager(auth): ExtractManager,
    Json(payload): Json<TagUpdateRequest>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::tag::do_update(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
