use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::icon_type::IconTypeUpdateRequest;
use _utils::models::{CommonResponse, common::EmptyResponse};

/// 修改分类
/// 由类型ID来定位修改一个分类
/// POST /icon_type/update
#[utoipa::path(
    post,
    path = "/api/icon_type/update",
    tag = "icon-type",
    summary = "修改图标分类",
    request_body = IconTypeUpdateRequest,
    responses(
        (status = 200, description = "更新结果", body = inline(CommonResponse<EmptyResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn update(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<IconTypeUpdateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::icon_type::do_update(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
