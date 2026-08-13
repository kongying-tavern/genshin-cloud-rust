use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use _utils::models::marker_link::{MarkerLinkGraphRequest, MarkerLinkListRequest};
use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};

/// 点位关联列表
/// POST /marker_link/get/list
#[tracing::instrument(skip(auth))]
pub async fn get_list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<MarkerLinkListRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // removed local alias
    match _functions::functions::api::marker_link::do_get_list(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 点位关联图数据
/// POST /marker_link/get/graph
#[tracing::instrument(skip(auth))]
pub async fn get_graph(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<MarkerLinkGraphRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::marker_link::do_get_graph(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
