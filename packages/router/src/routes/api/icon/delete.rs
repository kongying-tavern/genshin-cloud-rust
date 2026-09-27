use anyhow::Result;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractManager;

/// 删除图标
/// DELETE /icon/delete/{iconId}
#[tracing::instrument(skip(db, auth))]
pub async fn delete(
    State(db): State<crate::routes::SharedDb>,
    ExtractManager(auth): ExtractManager,
    Path(icon_id): Path<i64>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::icon::do_delete(db.as_ref(), auth, icon_id).await {
        Ok(resp) => Ok((StatusCode::OK, axum::Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
