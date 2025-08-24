use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::item::ItemUpdateData;

/// 修改物品
/// 提供修改同名物品功能，默认关闭
/// POST /item/update/{editSame}
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(edit_same): Path<bool>,
    Json(payload): Json<Vec<ItemUpdateData>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现修改物品的逻辑
    Ok(())
}
