use crate::middlewares::{ApiError, ExtractAuthInfo};
use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

/// 软删除标签类型
/// DELETE /tag_type/delete/{typeId}
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(type_id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::tag_type::do_delete(auth, type_id).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
