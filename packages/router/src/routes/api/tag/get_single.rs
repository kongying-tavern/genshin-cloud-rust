use anyhow::Result;

use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;
use _database::DB_CONN;

/// 按标签名查询单个标签（前端兼容路由）
/// POST /tag/get/single/{name}
#[tracing::instrument(skip(auth))]
pub async fn get_single(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(tag_name): Path<String>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::tag::do_get_single(DB_CONN.wait().as_ref(), auth, tag_name)
        .await
    {
        Ok(resp) => Ok((StatusCode::OK, axum::Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
