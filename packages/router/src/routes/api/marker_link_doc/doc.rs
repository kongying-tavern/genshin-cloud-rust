use crate::middlewares::ExtractAuthInfo;
use anyhow::Result;
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

/// 获取所有 marker_link 的二进制 md5 列表
/// GET /marker_link_doc/all-bin/md5
#[tracing::instrument(skip(auth))]
pub async fn all_bin_md5(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::marker_link_doc::do_all_list_bin_md5(
        auth,
        serde_json::json!({}),
    )
    .await
    {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}

/// 获取所有 marker_link 的二进制文件列表
/// GET /marker_link_doc/all-bin
#[tracing::instrument(skip(auth))]
pub async fn all_bin(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::marker_link_doc::do_all_list_bin(auth, serde_json::json!({}))
        .await
    {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}

/// 获取所有 marker_link 的图谱 md5 列表
/// GET /marker_link_doc/all-graph-bin/md5
#[tracing::instrument(skip(auth))]
pub async fn all_graph_bin_md5(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::marker_link_doc::do_all_graph_bin_md5(
        auth,
        serde_json::json!({}),
    )
    .await
    {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}

/// 获取所有 marker_link 的图谱二进制文件列表
/// GET /marker_link_doc/all-graph-bin
#[tracing::instrument(skip(auth))]
pub async fn all_graph_bin(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::marker_link_doc::do_all_graph_bin(auth, serde_json::json!({}))
        .await
    {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
