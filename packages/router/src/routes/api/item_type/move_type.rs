use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;

/// 批量移动类型为目标类型的子类型
/// 将类型批量移动到某个类型下作为其子类型
/// POST /item_type/move/{targetTypeId}
#[tracing::instrument(skip(auth))]
pub async fn move_to_target(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(target_type_id): Path<i64>,
    Json(payload): Json<Vec<i64>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现批量移动类型的逻辑
    Ok(())
}
