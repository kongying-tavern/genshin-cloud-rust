use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::tag::TagUpdateTypeRequest;

/// 修改标签的分类信息
/// 本接口仅在后台使用，故分离出来
/// POST /tag/update_type
#[tracing::instrument(skip(auth))]
pub async fn update_type(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<TagUpdateTypeRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::tag::do_update_type(auth, payload).await {
        Ok(_) => Ok((StatusCode::OK, Json(serde_json::json!({})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
