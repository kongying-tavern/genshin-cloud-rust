use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::marker_link::{MarkerLinkGraphRequest, MarkerLinkListRequest};

/// 点位关联列表
/// POST /marker_link/get/list
#[tracing::instrument(skip(auth))]
pub async fn get_list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<MarkerLinkListRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // removed local alias
    match _functions::functions::api::marker_link::do_get_list(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}

/// 点位关联图数据
/// POST /marker_link/get/graph
#[tracing::instrument(skip(auth))]
pub async fn get_graph(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<MarkerLinkGraphRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::marker_link::do_get_graph(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
