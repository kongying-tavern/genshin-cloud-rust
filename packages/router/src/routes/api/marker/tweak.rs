use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::marker::MarkerTweakRequest;

/// 点位调整
/// 批量调整点位数据，支持多种调整类型和字段
///
/// **注意事项**：
/// 1. 【额外数据】
///    - 部分包含多种数据字段与数据模型，批量修改时可能存在仅有部分点位允许设置某些字段的情况，建议前端对此进行检查和控制。
///    - 更新逻辑为：按字段更新
///      1. 字段为 `null`，删除字段
///      2. 字段为非 `null` 对象，替换原有值
///      3. 未传入的字段不变
/// 2. 【视频地址】无需进行接入，此功能仅为特殊用途开启的方案。
///
/// POST /marker/tweak
#[tracing::instrument(skip(auth))]
pub async fn tweak(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<MarkerTweakRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // TODO: 实现点位调整的逻辑
    Ok(())
}
