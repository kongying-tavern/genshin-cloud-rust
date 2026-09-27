use anyhow::Result;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractPunctuate;
use _utils::models::marker_link::MarkerLinkDeleteRequest;

/// 删除点位关联
/// DELETE /marker_link/delete
#[tracing::instrument(skip(db, auth))]
pub async fn delete(
    State(db): State<crate::routes::SharedDb>,
    ExtractPunctuate(auth): ExtractPunctuate,
    Json(payload): Json<MarkerLinkDeleteRequest>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::marker_link::do_delete(db.as_ref(), auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
