use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::item::ItemAddRequest;

/// 新增物品
/// 新建成功后会返回新物品ID
/// PUT /item/add
#[tracing::instrument(skip(auth))]
pub async fn add(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<ItemAddRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match crate::functions::api::item::do_add(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(serde_json::json!(v)))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
