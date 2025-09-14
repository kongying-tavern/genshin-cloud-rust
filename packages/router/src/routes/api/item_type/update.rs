use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::item_type::ItemTypeUpdateData;

/// 修改物品类型
/// POST /item_type/update
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<ItemTypeUpdateData>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match crate::functions::api::item_type::do_update(auth, payload).await {
        Ok(_) => Ok((StatusCode::OK, Json(serde_json::json!({})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
