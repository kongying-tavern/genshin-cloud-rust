use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::tag::TagUpdateTypeRequest;

/// 修改标签的分类信息（后台接口）
/// POST /tag/updateType
#[tracing::instrument(skip(auth, payload))]
pub async fn update_type(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<TagUpdateTypeRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::tag::do_update_type(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
