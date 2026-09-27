use anyhow::Result;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractManager;
use _utils::models::AreaAddRequest;

/// 新增地区
/// PUT /area/add
/// 返回新增地区ID
#[tracing::instrument(skip(db, auth))]
pub async fn add(
    State(db): State<crate::routes::SharedDb>,
    ExtractManager(auth): ExtractManager,
    Json(payload): Json<AreaAddRequest>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::area::do_add(db.as_ref(), auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
