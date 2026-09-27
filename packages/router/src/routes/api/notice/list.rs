use anyhow::Result;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::notice::NoticeListRequest;

/// 获取公告列表
#[tracing::instrument(skip(db, auth))]
pub async fn get_notice_list(
    State(db): State<crate::routes::SharedDb>,
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(request): Json<NoticeListRequest>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::notice::do_get_notice_list(db.as_ref(), auth, request).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
