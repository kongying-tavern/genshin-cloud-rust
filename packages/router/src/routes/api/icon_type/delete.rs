use anyhow::Result;

use crate::middlewares::{ApiError, ExtractAuthInfo};
use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use _utils::models::{CommonResponse, common::EmptyResponse};

/// 删除分类
/// 这个操作会递归删除，请在前端做二次确认
/// DELETE /icon_type/delete/{typeId}
#[utoipa::path(
    delete,
    path = "/api/icon_type/delete/{typeId}",
    tag = "icon-type",
    summary = "删除图标分类（递归删除）",
    params(("typeId" = i64, Path, description = "分类 ID")),
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
    match _functions::functions::api::icon_type::do_delete(auth, type_id).await {
        Ok(resp) => Ok((StatusCode::OK, axum::Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
