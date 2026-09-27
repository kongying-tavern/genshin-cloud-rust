use anyhow::Result;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::TagListRequest;

/// 标签列表
/// POST /tag/get/list
#[tracing::instrument(skip(db, auth))]
pub async fn list(
    State(db): State<crate::routes::SharedDb>,
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<TagListRequest>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::tag::do_list(db.as_ref(), auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
