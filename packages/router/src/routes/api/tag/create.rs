use _utils::models::CommonResponse;
use anyhow::Result;

use crate::middlewares::{ApiError, ExtractAuthInfo};
use axum::{extract::Path, http::StatusCode, response::IntoResponse};

/// 新增标签（前端兼容路由，仅传标签名）
/// PUT /tag/{tagName}
#[utoipa::path(
    put,
    path = "/api/tag/{tagName}",
    tag = "tag",
    summary = "按标签名新增标签（前端兼容路由）",
    params(("tagName" = String, Path, description = "标签名")),
    responses(
        (status = 200, description = "创建结果", body = inline(CommonResponse<bool>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn create(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(tag_name): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::tag::do_create_by_name(auth, tag_name).await {
        Ok(resp) => Ok((StatusCode::OK, axum::Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
