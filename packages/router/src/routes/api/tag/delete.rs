use crate::middlewares::{ApiError, ExtractAuthInfo};
use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use _utils::models::{CommonResponse, common::EmptyResponse};

/// 软删除标签
/// DELETE /tag/delete/{tagId}
#[utoipa::path(
    delete,
    path = "/api/tag/delete/{tagId}",
    tag = "tag",
    summary = "软删除标签",
    params(("tagId" = i64, Path, description = "标签 ID")),
    responses(
        (status = 200, description = "删除结果", body = inline(CommonResponse<EmptyResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(tag_id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::tag::do_delete(auth, tag_id).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
