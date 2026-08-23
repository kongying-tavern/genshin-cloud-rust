use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractManager;
use _utils::models::AreaAddRequest;

/// 新增地区
/// PUT /area/add
/// 返回新增地区ID
#[tracing::instrument(skip(auth))]
pub async fn add(
    ExtractManager(auth): ExtractManager,
    Json(payload): Json<AreaAddRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::area::do_add(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
