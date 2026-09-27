use anyhow::Result;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::marker_link::{MarkerLinkGraphRequest, MarkerLinkListRequest};

/// 点位关联列表
/// POST /marker_link/get/list
#[tracing::instrument(skip(db, auth))]
pub async fn get_list(
    State(db): State<crate::routes::SharedDb>,
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<MarkerLinkListRequest>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    // removed local alias
    match _functions::functions::api::marker_link::do_get_list(db.as_ref(), auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 点位关联图数据
/// POST /marker_link/get/graph
#[tracing::instrument(skip(db, auth))]
pub async fn get_graph(
    State(db): State<crate::routes::SharedDb>,
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<MarkerLinkGraphRequest>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::marker_link::do_get_graph(db.as_ref(), auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
