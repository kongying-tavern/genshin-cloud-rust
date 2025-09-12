use anyhow::Result;
use serde::{Deserialize, Serialize};

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::notice::NoticeAddRequest;

/// 新增公告
#[tracing::instrument(skip(auth))]
pub async fn add_notice(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(request): Json<NoticeAddRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::notice::do_add_notice(auth, request).await {
        Ok(id) => Ok((StatusCode::OK, Json(serde_json::json!({"id": id})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
