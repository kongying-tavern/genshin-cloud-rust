use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractManager;
use _utils::models::icon_type::IconTypeAddRequest;
use _utils::models::wrapper::CommonResponse;

/// 新增分类
/// 类型id在创建后返回
/// PUT /icon_type/add
#[tracing::instrument(skip(auth))]
pub async fn add(
    ExtractManager(auth): ExtractManager,
    Json(payload): Json<IconTypeAddRequest>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::icon_type::do_add(auth, payload).await {
        Ok(id) => Ok((StatusCode::OK, Json(CommonResponse::new(Ok(id))))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
