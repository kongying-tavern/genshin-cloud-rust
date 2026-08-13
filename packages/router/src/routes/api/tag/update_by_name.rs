use _utils::models::CommonResponse;
use anyhow::Result;

use crate::middlewares::{ApiError, ExtractAuthInfo};
use axum::{extract::Path, http::StatusCode, response::IntoResponse};

/// 按标签名更新图标绑定（前端兼容路由）
/// POST /tag/{tagName}/{iconId}
#[utoipa::path(
    post,
    path = "/api/tag/{tagName}/{iconId}",
    tag = "tag",
    summary = "按标签名更新图标绑定（前端兼容路由）",
    params(
        ("tagName" = String, Path, description = "标签名"),
        ("iconId" = i64, Path, description = "图标 ID"),
    ),
    responses(
        (status = 200, description = "更新结果", body = inline(CommonResponse<bool>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path((tag_name, icon_id)): Path<(String, i64)>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::tag::do_update_by_name(auth, tag_name, icon_id).await {
        Ok(resp) => Ok((StatusCode::OK, axum::Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
