use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::icon::IconListRequest;
use _utils::models::{CommonResponse, IconListResponse};

/// 列出图标
/// 可按照分类（分类需保证为末端分类）和上传者进行查询，也可根据ID批量查询，可分页
/// POST /icon/get/list
#[utoipa::path(
    post,
    path = "/api/icon/get/list",
    tag = "icon",
    summary = "列出图标",
    request_body = IconListRequest,
    responses(
        (status = 200, description = "图标分页列表", body = inline(CommonResponse<IconListResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<IconListRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::icon::do_list(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
