use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;

/// 删除地区公用物品
/// DELETE /item_common/delete/{itemId}
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(item_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match crate::functions::api::item_common::do_delete(auth, item_id).await {
        Ok(_) => Ok((StatusCode::OK, Json(serde_json::json!({})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
