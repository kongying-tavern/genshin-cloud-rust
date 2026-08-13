use crate::middlewares::{ApiError, ExtractAuthInfo};
use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use _utils::models::{CommonResponse, MarkerEmptyResponse};

/// 删除点位
/// DELETE /marker/{markerId}
#[utoipa::path(
    delete,
    path = "/api/marker/{marker_id}",
    tag = "marker",
    summary = "删除点位",
    params(("marker_id" = i64, Path, description = "点位 ID")),
    responses(
        (status = 200, description = "删除结果", body = inline(CommonResponse<MarkerEmptyResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn delete(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(marker_id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::marker::do_delete(auth, marker_id).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
