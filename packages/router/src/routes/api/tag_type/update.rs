use anyhow::Result;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractManager;
use _utils::models::TagTypeUpdateRequest;

/// 更新标签类型
/// POST /tag_type/update
#[tracing::instrument(skip(db, auth))]
pub async fn update(
    State(db): State<crate::routes::SharedDb>,
    ExtractManager(auth): ExtractManager,
    Json(payload): Json<TagTypeUpdateRequest>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::tag_type::do_update(db.as_ref(), auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
