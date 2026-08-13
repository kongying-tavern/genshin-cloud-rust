use _utils::models::CommonResponse;
use anyhow::Result;

use crate::middlewares::{ApiError, ExtractAuthInfo};
use axum::{extract::Path, http::StatusCode, response::IntoResponse};

/// 删除图标
/// DELETE /icon/delete/{iconId}
#[utoipa::path(
    delete,
    path = "/api/icon/delete/{iconId}",
    tag = "icon",
    summary = "删除图标",
    params(("iconId" = i64, Path, description = "图标 ID")),
    responses(
        (status = 200, description = "删除结果", body = inline(CommonResponse<utoipa::TupleUnit>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(icon_id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::icon::do_delete(auth, icon_id).await {
        Ok(resp) => Ok((StatusCode::OK, axum::Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
