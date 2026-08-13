use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use _utils::models::icon::IconUpdateRequest;
use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};

/// 修改图标信息
/// 由icon_id定位修改一个icon
/// POST /icon/update
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<IconUpdateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::icon::do_update(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
