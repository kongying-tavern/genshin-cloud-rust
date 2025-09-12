use anyhow::Result;
use serde::{Deserialize, Serialize};

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::{wrapper::Pagination, punctuate_audit::PunctuateAuditFilterRequest};

/// 根据各种条件筛选打点ID
/// POST /punctuate_audit/get/id
#[tracing::instrument(skip(auth))]
pub async fn get_id(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<PunctuateAuditFilterRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::punctuate_audit::do_get_list_by_info(
        auth,
        payload,
    )
    .await
    {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}

/// 根据各种条件筛选打点信息
/// POST /punctuate_audit/get/list_byinfo
#[tracing::instrument(skip(auth))]
pub async fn get_list_by_info(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<PunctuateAuditFilterRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::punctuate_audit::do_get_list_by_info(
        auth,
        payload,
    )
    .await
    {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}

/// 通过打点ID列表查询打点信息
/// POST /punctuate_audit/get/list_byid
#[tracing::instrument(skip(auth))]
pub async fn get_list_by_id(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<Vec<i64>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Pass the Vec<i64> as-is to functions layer
    match _functions::functions::api::punctuate_audit::do_get_list_by_id(
        auth,
        payload,
    )
    .await
    {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}

/// 分页查询所有打点信息（包括暂存）
/// POST /punctuate_audit/get/page/all
#[tracing::instrument(skip(auth))]
pub async fn get_page_all(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<Pagination>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::punctuate_audit::do_get_page_all(
        auth,
        payload,
    )
    .await
    {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
