use _utils::models::CommonResponse;
use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::AreaAddRequest;

/// 新增地区
/// PUT /area/add
/// 返回新增地区ID
#[utoipa::path(
    put,
    path = "/api/area/add",
    tag = "area",
    summary = "新增地区",
    request_body = AreaAddRequest,
    responses(
        (status = 200, description = "新增地区 ID", body = inline(CommonResponse<i64>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn add(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<AreaAddRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::area::do_add(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
