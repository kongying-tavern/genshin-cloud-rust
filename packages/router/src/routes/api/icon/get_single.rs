use anyhow::Result;

use crate::middlewares::{ApiError, ExtractAuthInfo};
use axum::{extract::Path, http::StatusCode, response::IntoResponse};

/// 获取单个图标信息
/// POST /icon/get/single/{iconId}
#[tracing::instrument(skip(auth))]
pub async fn get_single(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(icon_id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::icon::do_get_single(auth, icon_id).await {
        Ok(resp) => Ok((StatusCode::OK, axum::Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
