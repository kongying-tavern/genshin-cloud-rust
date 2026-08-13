use _utils::models::CommonResponse;
use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::icon_type::IconTypeAddRequest;

/// 新增分类
/// 类型id在创建后返回
/// PUT /icon_type/add
#[utoipa::path(
    put,
    path = "/api/icon_type/add",
    tag = "icon-type",
    summary = "新增图标分类",
    request_body = IconTypeAddRequest,
    responses(
        (status = 200, description = "新增分类 ID", body = inline(CommonResponse<i64>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn add(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<IconTypeAddRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::icon_type::do_add(auth, payload).await {
        Ok(id) => Ok((StatusCode::OK, Json(CommonResponse::new(Ok(id))))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
