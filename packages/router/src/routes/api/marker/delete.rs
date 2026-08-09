use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;

/// 删除点位
/// DELETE /marker/{markerId}
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(marker_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::marker::do_delete(auth, marker_id).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
