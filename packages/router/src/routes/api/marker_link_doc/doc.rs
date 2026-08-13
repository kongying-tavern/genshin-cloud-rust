use crate::middlewares::{ApiError, ExtractAuthInfo};
use _functions::functions::api::binary_doc::BinaryMd5Vo;
use _utils::models::CommonResponse;
use anyhow::Result;
use axum::{
    body::Bytes,
    extract::Json,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};

/// 获取所有 marker_link 的二进制 md5
/// GET /marker_link_doc/all-bin/md5
#[utoipa::path(
    get,
    path = "/api/marker_link_doc/all_list_bin_md5",
    tag = "marker-link-doc",
    summary = "获取所有 marker_link 的二进制 MD5",
    responses(
        (status = 200, description = "MD5 信息", body = inline(CommonResponse<BinaryMd5Vo>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn all_bin_md5(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::marker_link_doc::do_all_list_bin_md5(
        auth,
        serde_json::json!({}),
    )
    .await
    {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 获取所有 marker_link 的二进制文件（GZIP 压缩）
/// GET /marker_link_doc/all-bin
#[utoipa::path(
    get,
    path = "/api/marker_link_doc/all_list_bin",
    tag = "marker-link-doc",
    summary = "获取所有 marker_link 的二进制文件（GZIP 压缩）",
    responses(
        (status = 200, description = "GZIP 压缩二进制", content_type = "application/octet-stream", body = Vec<u8>),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn all_bin(ExtractAuthInfo(auth): ExtractAuthInfo) -> Result<Response, ApiError> {
    match _functions::functions::api::marker_link_doc::do_all_list_bin(auth).await {
        Ok(bytes) => Ok((
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "application/octet-stream"),
                (header::CACHE_CONTROL, "no-store"),
            ],
            Bytes::from(bytes),
        )
            .into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 获取所有 marker_link 的图谱 md5
/// GET /marker_link_doc/all-graph-bin/md5
#[utoipa::path(
    get,
    path = "/api/marker_link_doc/all_graph_bin_md5",
    tag = "marker-link-doc",
    summary = "获取所有 marker_link 的图谱 MD5",
    responses(
        (status = 200, description = "MD5 信息", body = inline(CommonResponse<BinaryMd5Vo>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn all_graph_bin_md5(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::marker_link_doc::do_all_graph_bin_md5(
        auth,
        serde_json::json!({}),
    )
    .await
    {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 获取所有 marker_link 的图谱二进制文件（GZIP 压缩）
/// GET /marker_link_doc/all-graph-bin
#[utoipa::path(
    get,
    path = "/api/marker_link_doc/all_graph_bin",
    tag = "marker-link-doc",
    summary = "获取所有 marker_link 的图谱二进制文件（GZIP 压缩）",
    responses(
        (status = 200, description = "GZIP 压缩二进制", content_type = "application/octet-stream", body = Vec<u8>),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn all_graph_bin(ExtractAuthInfo(auth): ExtractAuthInfo) -> Result<Response, ApiError> {
    match _functions::functions::api::marker_link_doc::do_all_graph_bin(auth).await {
        Ok(bytes) => Ok((
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "application/octet-stream"),
                (header::CACHE_CONTROL, "no-store"),
            ],
            Bytes::from(bytes),
        )
            .into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
