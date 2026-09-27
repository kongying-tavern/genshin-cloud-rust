use anyhow::Result;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractManager;
use _utils::models::TagAddRequest;

/// 新增标签
/// PUT /tag/add
/// 返回新增标签 ID
#[tracing::instrument(skip(db, auth))]
pub async fn add(
    State(db): State<crate::routes::SharedDb>,
    ExtractManager(auth): ExtractManager,
    Json(payload): Json<TagAddRequest>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::tag::do_add(db.as_ref(), auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(serde_json::json!({"id": resp.id})))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
