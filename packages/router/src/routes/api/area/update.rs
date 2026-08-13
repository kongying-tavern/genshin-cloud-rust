use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::{AreaUpdateRequest, common::EmptyResponse, wrapper::CommonResponse};

/// 修改地区
/// POST /area/update
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<AreaUpdateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::area::do_update(auth, payload).await {
        Ok(_) => Ok(Json(CommonResponse::new(Ok(EmptyResponse {}))).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
