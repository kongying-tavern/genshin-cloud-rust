use anyhow::Result;
use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;

/// 将物品加入某一类型
/// 根据物品ID列表批量加入
/// POST /item/join/{typeId}
#[tracing::instrument(skip_all)]
pub async fn join_type(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(type_id): Path<i64>,
    Json(payload): Json<Vec<i64>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现将物品加入类型的逻辑
    Ok(())
}
