use anyhow::Result;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAdmin;

/// 删除全部物品缓存
#[tracing::instrument(skip(db, auth))]
pub async fn delete_item_cache(
    State(db): State<crate::routes::SharedDb>,
    ExtractAdmin(auth): ExtractAdmin,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match _functions::functions::api::cache::do_delete_item_cache(db.as_ref(), auth).await {
        Ok(resp) => Ok((StatusCode::OK, Json(resp))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
