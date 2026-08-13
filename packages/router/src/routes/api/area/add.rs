use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use _utils::models::AreaAddRequest;
use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};

/// 新增地区
/// PUT /area/add
/// 返回新增地区ID
#[tracing::instrument(skip(auth))]
pub async fn add(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<AreaAddRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::area::do_add(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
