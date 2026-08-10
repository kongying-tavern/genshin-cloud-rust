use anyhow::Result;

use axum::{extract::Json, extract::Path, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::{common::EmptyResponse, wrapper::CommonResponse};

/// 删除物品类型
/// 批量递归删除物品类型，需在前端做二次确认
/// DELETE /item_type/delete/{itemTypeId}
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(item_type_id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::item_type::do_delete(auth, item_type_id).await {
        Ok(_) => Ok(Json(CommonResponse::new(Ok(EmptyResponse {}))).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
