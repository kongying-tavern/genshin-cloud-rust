use anyhow::Result;

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractPunctuate;

/// 删除点位
/// DELETE /marker/{markerId}
#[tracing::instrument(skip(db, auth))]
pub async fn delete(
    State(db): State<crate::routes::SharedDb>,
    ExtractPunctuate(auth): ExtractPunctuate,
    Path(marker_id): Path<i64>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::marker::do_delete(db.as_ref(), auth, marker_id).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
