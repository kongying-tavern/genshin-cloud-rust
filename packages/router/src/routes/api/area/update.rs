use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractManager;
use _utils::models::{AreaUpdateRequest, common::EmptyResponse, wrapper::CommonResponse};

/// 修改地区
/// POST /area/update
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractManager(auth): ExtractManager,
    Json(payload): Json<AreaUpdateRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::area::do_update(auth, payload).await {
        Ok(_) => Ok(Json(CommonResponse::new(Ok(EmptyResponse {}))).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
