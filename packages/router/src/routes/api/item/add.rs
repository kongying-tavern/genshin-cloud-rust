use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::item::ItemAddRequest;

/// 新增物品
/// 新建成功后会返回新物品ID
/// PUT /item/add
#[tracing::instrument(skip(auth))]
pub async fn add(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<ItemAddRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match crate::functions::api::item::do_add(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
