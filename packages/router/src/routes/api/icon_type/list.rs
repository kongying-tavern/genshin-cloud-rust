use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::icon_type::IconTypeListRequest;

/// 列出分类
/// 列出图标的分类，typeId为-1的时候为列出所有的根分类
/// POST /icon_type/get/list
#[tracing::instrument(skip(auth))]
pub async fn list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<IconTypeListRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::icon_type::do_list(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, axum::Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
