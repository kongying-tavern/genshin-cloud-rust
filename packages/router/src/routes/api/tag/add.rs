use _utils::models::TagAddResponse;
use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::TagAddRequest;

/// 新增标签
/// PUT /tag/add
/// 返回新增标签 ID
#[utoipa::path(
    put,
    path = "/api/tag/add",
    tag = "tag",
    summary = "新增标签",
    request_body = TagAddRequest,
    responses(
        (status = 200, description = "新增标签 ID", body = TagAddResponse),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn add(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<TagAddRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::tag::do_add(auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, Json(serde_json::json!({"id": resp.id})))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
