use _utils::models::CommonResponse;
use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::icon::IconAddRequest;

/// 新增图标
/// 无需指定icon的id，id由系统自动生成并在响应中返回
/// 一组name和creator需要唯一（允许单一重复）
/// PUT /icon/add
#[utoipa::path(
    put,
    path = "/api/icon/add",
    tag = "icon",
    summary = "新增图标",
    request_body = IconAddRequest,
    responses(
        (status = 200, description = "新增图标 ID", body = inline(CommonResponse<i64>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn add(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<IconAddRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::icon::do_add(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
