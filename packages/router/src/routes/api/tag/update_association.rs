use anyhow::Result;

use axum::extract::Json;
use axum::{extract::Path, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 修改标签关联
/// POST /tag/{tagName}/{iconId}
#[tracing::instrument(skip(auth))]
pub async fn update_association(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path((tag_name, icon_id)): Path<(String, i64)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::tag::do_update_association(
        auth,
        serde_json::json!({"tag_name": tag_name, "icon_id": icon_id}),
    )
    .await
    {
        Ok(_) => Ok((StatusCode::OK, Json(serde_json::json!({})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
