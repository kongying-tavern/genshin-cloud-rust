use anyhow::Result;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::icon_type::IconTypeListRequest;

/// 列出分类
/// 列出图标的分类，typeId为-1的时候为列出所有的根分类
/// POST /icon_type/get/list
#[tracing::instrument(skip(db, auth))]
pub async fn list(
    State(db): State<crate::routes::SharedDb>,
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Json(payload): Json<IconTypeListRequest>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::icon_type::do_list(db.as_ref(), auth, payload).await {
        Ok(resp) => Ok((StatusCode::OK, axum::Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
