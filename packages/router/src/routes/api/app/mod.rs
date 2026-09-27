use anyhow::Result;

use axum::{
    Router,
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
};

use crate::middlewares::ExtractAdmin;

/// 触发应用更新（清空 BinaryMD5 缓存，客户端下次轮询重新拉取）
/// POST /app/trigger/update
#[tracing::instrument(skip(db, auth))]
pub async fn trigger_update(
    State(db): State<crate::routes::SharedDb>,
    ExtractAdmin(auth): ExtractAdmin,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::app::do_trigger_update(db.as_ref(), auth).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

pub async fn router() -> Result<Router<crate::routes::SharedDb>> {
    let ret = Router::new().route("/trigger/update", post(trigger_update));
    Ok(ret)
}
