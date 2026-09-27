use anyhow::Result;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::AreaListRequest;

/// 列出地区
/// POST /area/get/list
/// 可根据父级地区id列出子地区列表
#[tracing::instrument(skip(db, auth))]
pub async fn list(
    State(db): State<crate::routes::SharedDb>,
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<AreaListRequest>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::area::do_list(db.as_ref(), auth, payload).await {
        Ok(list) => Ok((StatusCode::OK, Json(list))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
