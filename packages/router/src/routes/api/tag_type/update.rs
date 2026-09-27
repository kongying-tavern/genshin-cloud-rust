use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractManager;
use _database::DB_CONN;
use _utils::models::TagTypeUpdateRequest;

/// 更新标签类型
/// POST /tag_type/update
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractManager(auth): ExtractManager,
    Json(payload): Json<TagTypeUpdateRequest>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::tag_type::do_update(DB_CONN.wait().as_ref(), auth, payload)
        .await
    {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
