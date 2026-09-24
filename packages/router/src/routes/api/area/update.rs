use anyhow::Result;

use axum::{extract::Json, response::IntoResponse};

use crate::middlewares::ExtractManager;
use _utils::models::AreaUpdateRequest;

/// 修改地区
/// POST /area/update
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractManager(auth): ExtractManager,
    Json(payload): Json<AreaUpdateRequest>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::area::do_update(auth, payload).await {
        Ok(resp) => Ok(Json(resp).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
