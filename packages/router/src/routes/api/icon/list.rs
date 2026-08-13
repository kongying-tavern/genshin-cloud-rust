use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use _utils::models::icon::IconListRequest;
use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};

/// 列出图标
/// 可按照分类（分类需保证为末端分类）和上传者进行查询，也可根据ID批量查询，可分页
/// POST /icon/get/list
#[tracing::instrument(skip(auth))]
pub async fn list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<IconListRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::icon::do_list(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
