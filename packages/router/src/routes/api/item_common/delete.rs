use anyhow::Result;
use crate::middlewares::{ApiError, ExtractAuthInfo};

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};


/// 删除地区公用物品
/// DELETE /item_common/delete/{itemId}
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(item_id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    match crate::functions::api::item_common::do_delete(auth, item_id).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
