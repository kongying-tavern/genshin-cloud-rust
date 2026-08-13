use _utils::models::CommonResponse;
use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::icon::IconUpdateRequest;

/// 修改图标信息
/// 由icon_id定位修改一个icon
/// POST /icon/update
#[utoipa::path(
    post,
    path = "/api/icon/update",
    tag = "icon",
    summary = "修改图标信息",
    request_body = IconUpdateRequest,
    responses(
        (status = 200, description = "更新结果", body = inline(CommonResponse<utoipa::TupleUnit>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<IconUpdateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::icon::do_update(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
