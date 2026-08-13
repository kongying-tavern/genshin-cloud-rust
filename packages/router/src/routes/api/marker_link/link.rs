use _utils::models::CommonResponse;
use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::marker_link::MarkerLinkage;

/// 关联点位
/// POST /marker_link/link
#[utoipa::path(
    post,
    path = "/api/marker_link/link",
    tag = "marker-link",
    summary = "关联点位",
    request_body = Vec<MarkerLinkage>,
    responses(
        (status = 200, description = "关联结果", body = inline(CommonResponse<String>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn link(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<Vec<MarkerLinkage>>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::marker_link::do_link(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
