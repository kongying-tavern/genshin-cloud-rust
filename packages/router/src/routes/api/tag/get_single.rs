use anyhow::Result;

use crate::middlewares::{ApiError, ExtractAuthInfo};
use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use _utils::models::{CommonResponse, TagVO};

/// 按标签名查询单个标签（前端兼容路由）
/// POST /tag/get/single/{name}
#[utoipa::path(
    post,
    path = "/api/tag/get/single/{name}",
    tag = "tag",
    summary = "按标签名查询单个标签",
    params(("name" = String, Path, description = "标签名")),
    responses(
        (status = 200, description = "标签信息", body = inline(CommonResponse<TagVO>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn get_single(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(tag_name): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::tag::do_get_single(auth, tag_name).await {
        Ok(resp) => Ok((StatusCode::OK, axum::Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
