use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
// duplicate import removed

use crate::middlewares::ExtractAuthInfo;
use _utils::models::{punctuate::PunctuateData, wrapper::Pagination};

/// 分页查询所有打点信息
/// POST /punctuate/get/page
#[tracing::instrument(skip(auth))]
pub async fn get_page(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<Pagination>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::punctuate::do_get_page(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}

/// 分页查询自己提交的未通过的打点信息
/// POST /punctuate/get/page/{authorId}
#[tracing::instrument(skip(auth))]
pub async fn get_page_by_author(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(author_id): Path<i64>,
    Json(payload): Json<Pagination>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::punctuate::do_get_page_by_author(auth, author_id, payload)
        .await
    {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
