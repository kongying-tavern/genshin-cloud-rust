use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use _utils::models::AreaListRequest;
use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};

/// 列出地区
/// POST /area/get/list
/// 可根据父级地区id列出子地区列表
#[tracing::instrument(skip(auth))]
pub async fn list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<AreaListRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::area::do_list(auth, payload).await {
        Ok(list) => Ok((StatusCode::OK, Json(list))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
