use crate::middlewares::{ApiError, ExtractAuthInfo};
use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use _utils::models::{CommonResponse, common::EmptyResponse};

/// 软删除标签类型
/// DELETE /tag_type/delete/{typeId}
#[utoipa::path(
    delete,
    path = "/api/tag_type/delete/{typeId}",
    tag = "tag-type",
    summary = "软删除标签类型",
    params(("typeId" = i64, Path, description = "标签类型 ID")),
    responses(
        (status = 200, description = "删除结果", body = inline(CommonResponse<EmptyResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(type_id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::tag_type::do_delete(auth, type_id).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
