use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractManager;
use _utils::models::{common::EmptyResponse, item::ItemUpdateData, wrapper::CommonResponse};

/// 修改物品
/// 提供修改同名物品功能，默认关闭
/// POST /item/update/{editSame}
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractManager(auth): ExtractManager,
    Path(edit_same): Path<i64>,
    Json(payload): Json<Vec<ItemUpdateData>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::item::do_update(auth, edit_same != 0, payload).await {
        Ok(_) => Ok(Json(CommonResponse::new(Ok(EmptyResponse {}))).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
