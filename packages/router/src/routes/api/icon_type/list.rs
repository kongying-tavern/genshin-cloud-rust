use anyhow::Result;

use axum::{http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::icon_type::IconTypeListRequest;
use _utils::models::{CommonResponse, IconTypeListResponse};

/// 列出分类
/// 列出图标的分类，typeId为-1的时候为列出所有的根分类
/// POST /icon_type/get/list
#[utoipa::path(
    post,
    path = "/api/icon_type/get/list",
    tag = "icon-type",
    summary = "列出图标分类",
    request_body = IconTypeListRequest,
    responses(
        (status = 200, description = "分类分页列表", body = inline(CommonResponse<IconTypeListResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<IconTypeListRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::icon_type::do_list(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, axum::Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
