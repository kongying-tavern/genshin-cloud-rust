use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::icon_type::IconTypeAddRequest;

/// 新增分类
/// 类型id在创建后返回
/// PUT /icon_type/add
#[tracing::instrument(skip_all)]
pub async fn add(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<IconTypeAddRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(())
}
