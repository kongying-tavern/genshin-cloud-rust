use anyhow::Result;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

/// 获取资源信息（去重上传第一跳）
/// GET /res/get?filePath=...
#[tracing::instrument(skip(db))]
pub async fn get(
    State(db): State<crate::routes::SharedDb>,
    Query(query): Query<GetResourceQuery>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::res::do_get(db.as_ref(), query.file_path).await {
        Ok(v) => Ok((StatusCode::OK, axum::Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 查询参数（Java `getResource(String filePath)`：可缺省）。
#[derive(Debug, Deserialize)]
pub struct GetResourceQuery {
    #[serde(rename = "filePath")]
    pub file_path: Option<String>,
}
