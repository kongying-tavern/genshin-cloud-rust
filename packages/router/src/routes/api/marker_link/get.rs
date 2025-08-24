use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::marker_link::{MarkerLinkGraphRequest, MarkerLinkListRequest};

/// 点位关联列表
/// POST /marker_link/get/list
#[tracing::instrument(skip_all)]
pub async fn get_list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<MarkerLinkListRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现获取点位关联列表的逻辑
    Ok(())
}

/// 点位关联图数据
/// POST /marker_link/get/graph
#[tracing::instrument(skip_all)]
pub async fn get_graph(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<MarkerLinkGraphRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现获取点位关联图数据的逻辑
    Ok(())
}
