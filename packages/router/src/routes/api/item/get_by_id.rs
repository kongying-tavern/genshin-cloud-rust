use anyhow::Result;
use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;

/// 根据物品ID查询物品
/// 输入ID列表查询，单个查询也用此API
/// POST /item/get/list_by_id
#[tracing::instrument(skip_all)]
pub async fn get_list_by_id(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<Vec<i64>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现根据ID列表查询物品的逻辑
    Ok(())
}
