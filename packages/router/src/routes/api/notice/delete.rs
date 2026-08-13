use _utils::models::CommonResponse;
use anyhow::Result;

use crate::middlewares::{ApiError, ExtractAuthInfo};
use axum::extract::Json;
use axum::{extract::Path, response::IntoResponse};

/// 删除公告
#[utoipa::path(
    delete,
    path = "/api/notice/{noticeId}",
    tag = "notice",
    summary = "删除公告",
    params(("noticeId" = i64, Path, description = "公告 ID")),
    responses(
        (status = 200, description = "删除结果", body = inline(CommonResponse<utoipa::TupleUnit>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn delete_notice(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(notice_id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::notice::do_delete_notice(auth, notice_id).await {
        Ok(v) => Ok(Json(v).into_response()),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
