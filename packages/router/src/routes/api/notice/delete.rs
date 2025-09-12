use anyhow::Result;

use axum::extract::Json;
use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 删除公告
#[tracing::instrument(skip(auth))]
pub async fn delete_notice(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(notice_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::notice::do_delete_notice(auth, notice_id).await {
        Ok(_) => Ok((StatusCode::OK, Json(serde_json::json!({})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
