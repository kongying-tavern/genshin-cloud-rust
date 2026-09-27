use anyhow::Result;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractPunctuate;
use _utils::models::marker_link::MarkerLinkage;

/// 关联点位
/// POST /marker_link/link
#[tracing::instrument(skip(db, auth))]
pub async fn link(
    State(db): State<crate::routes::SharedDb>,
    ExtractPunctuate(auth): ExtractPunctuate,
    Json(payload): Json<Vec<MarkerLinkage>>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::marker_link::do_link(db.as_ref(), auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
