use anyhow::Result;

use crate::middlewares::{ApiError, ExtractAuthInfo};
use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use _utils::models::{CommonResponse, IconVO};

/// 获取单个图标信息
/// POST /icon/get/single/{iconId}
#[utoipa::path(
    post,
    path = "/api/icon/get/single/{iconId}",
    tag = "icon",
    summary = "获取单个图标信息",
    params(("iconId" = i64, Path, description = "图标 ID")),
    responses(
        (status = 200, description = "图标信息", body = inline(CommonResponse<IconVO>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn get_single(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(icon_id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::icon::do_get_single(auth, icon_id).await {
        Ok(resp) => Ok((StatusCode::OK, axum::Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
