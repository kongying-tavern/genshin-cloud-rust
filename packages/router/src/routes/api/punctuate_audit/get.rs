use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::{punctuate_audit::PunctuateAuditFilterRequest, wrapper::Pagination};

/// 根据提交者列表查询待审核打点 ID
/// POST /punctuate_audit/get/id
#[tracing::instrument(skip(auth))]
pub async fn get_id(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<PunctuateAuditFilterRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let authors = payload.author_list.unwrap_or_default();
    match _functions::functions::api::punctuate_audit::do_get_list_by_authors(auth, authors).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}

/// 根据提交者列表查询待审核打点信息
/// POST /punctuate_audit/get/list_byinfo
#[tracing::instrument(skip(auth))]
pub async fn get_list_by_info(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<PunctuateAuditFilterRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let authors = payload.author_list.unwrap_or_default();
    match _functions::functions::api::punctuate_audit::do_get_list_by_authors(auth, authors).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}

/// 通过打点 ID 列表查询打点信息
/// POST /punctuate_audit/get/list_byid
#[tracing::instrument(skip(auth))]
pub async fn get_list_by_id(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<Vec<i64>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::punctuate_audit::do_get_list_by_id(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}

/// 分页查询所有待审核打点信息
/// POST /punctuate_audit/get/page/all
#[tracing::instrument(skip(auth))]
pub async fn get_page_all(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<Pagination>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::punctuate_audit::do_get_page_all(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}
