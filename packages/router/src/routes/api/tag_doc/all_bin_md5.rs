use _utils::models::CommonResponse;
use anyhow::Result;

use crate::middlewares::{ApiError, ExtractAuthInfo};
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use _functions::functions::api::binary_doc::BinaryMd5Vo;

/// 获取所有标签信息的 MD5
/// GET /tag_doc/all_bin_md5
#[utoipa::path(
    get,
    path = "/api/tag_doc/all_bin_md5",
    tag = "tag-doc",
    summary = "获取所有标签信息的 MD5",
    responses(
        (status = 200, description = "MD5 信息", body = inline(CommonResponse<BinaryMd5Vo>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn all_bin_md5(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::tag_doc::do_all_bin_md5(auth, serde_json::json!({})).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
