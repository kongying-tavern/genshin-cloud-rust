use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::icon::IconAddRequest;

/// 新增图标
/// 无需指定icon的id，id由系统自动生成并在响应中返回
/// 一组name和creator需要唯一（允许单一重复）
/// PUT /icon/add
#[tracing::instrument(skip_all)]
pub async fn add(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<IconAddRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
