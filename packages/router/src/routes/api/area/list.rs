use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::AreaListRequest;
use _utils::models::{AreaListResponse, CommonResponse};

/// 列出地区
/// POST /area/get/list
/// 可根据父级地区id列出子地区列表
#[utoipa::path(
    post,
    path = "/api/area/get/list",
    tag = "area",
    summary = "列出地区",
    request_body = AreaListRequest,
    responses(
        (status = 200, description = "地区列表", body = inline(CommonResponse<AreaListResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<AreaListRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::area::do_list(auth, payload).await {
        Ok(list) => Ok((StatusCode::OK, Json(list))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
