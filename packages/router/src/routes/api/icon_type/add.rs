use anyhow::Result;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractManager;
use _utils::models::icon_type::IconTypeAddRequest;
use _utils::models::wrapper::CommonResponse;

/// 新增分类
/// 类型id在创建后返回
/// PUT /icon_type/add
#[tracing::instrument(skip(db, auth))]
pub async fn add(
    State(db): State<crate::routes::SharedDb>,
    ExtractManager(auth): ExtractManager,
    Json(payload): Json<IconTypeAddRequest>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::icon_type::do_add(db.as_ref(), auth, payload).await {
        Ok(id) => Ok((StatusCode::OK, Json(CommonResponse::new(Ok(id))))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
