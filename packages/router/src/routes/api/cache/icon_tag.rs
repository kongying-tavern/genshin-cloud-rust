use anyhow::Result;

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use axum::{http::StatusCode, response::IntoResponse};

use _utils::models::{CommonResponse, common::EmptyResponse};

/// 删除标签缓存
/// DELETE /cache/icon_tag
/// body 为要清除的 tag 列表（`string[]`，list 为空则删除所有标签缓存）
#[utoipa::path(
    delete,
    path = "/api/cache/icon_tag",
    tag = "cache",
    summary = "删除标签缓存",
    description = "body 为要清除的 tag 列表（空数组则删除所有标签缓存）；兼容 camelCase 路径 /api/cache/iconTag",
    request_body = Vec<String>,
    responses(
        (status = 200, description = "删除结果", body = inline(CommonResponse<EmptyResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn delete_icon_tag_cache(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(tags): AppJson<Vec<String>>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::cache::do_delete_icon_tag_cache(auth, tags).await {
        Ok(resp) => Ok((StatusCode::OK, axum::Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
