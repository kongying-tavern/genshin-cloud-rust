use anyhow::Result;
use axum::{
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;

/// 点位关联列表数据md5
/// GET /marker_link_doc/all_list_bin_md5
#[tracing::instrument(skip_all)]
pub async fn all_list_bin_md5(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现获取点位关联列表数据md5的逻辑
    Ok(())
}

/// 点位关联列表数据
/// GET /marker_link_doc/all_list_bin
#[tracing::instrument(skip_all)]
pub async fn all_list_bin(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现获取点位关联列表数据的逻辑
    Ok(())
}

/// 点位关联有向图数据md5
/// GET /marker_link_doc/all_graph_bin_md5
#[tracing::instrument(skip_all)]
pub async fn all_graph_bin_md5(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现获取点位关联有向图数据md5的逻辑
    Ok(())
}

/// 点位关联有向图数据
/// GET /marker_link_doc/all_graph_bin
#[tracing::instrument(skip_all)]
pub async fn all_graph_bin(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现获取点位关联有向图数据的逻辑
    Ok(())
}
