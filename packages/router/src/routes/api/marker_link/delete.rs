use _utils::models::CommonResponse;
use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::marker_link::MarkerLinkDeleteRequest;

/// 删除点位关联
/// DELETE /marker_link/delete
#[utoipa::path(
    delete,
    path = "/api/marker_link/delete",
    tag = "marker-link",
    summary = "删除点位关联",
    request_body = MarkerLinkDeleteRequest,
    responses(
        (status = 200, description = "删除结果", body = inline(CommonResponse<serde_json::Value>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<MarkerLinkDeleteRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::marker_link::do_delete(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
