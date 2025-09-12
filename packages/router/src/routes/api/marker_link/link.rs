use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::marker_link::MarkerLinkage;

/// 关联点位
/// POST /marker_link/link
#[tracing::instrument(skip(auth))]
pub async fn link(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<Vec<MarkerLinkage>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::marker_link::do_link(
        auth,
        payload,
    )
    .await
    {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
