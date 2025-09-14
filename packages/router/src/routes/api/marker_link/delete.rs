use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::marker_link::MarkerLinkDeleteRequest;

/// 删除点位关联
/// DELETE /marker_link/delete
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<MarkerLinkDeleteRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::marker_link::do_delete(auth, payload).await {
        Ok(_) => Ok((StatusCode::OK, Json(serde_json::json!({})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
