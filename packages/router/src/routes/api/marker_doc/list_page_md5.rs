use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::ExtractAuthInfo;

/// 点位分页的md5数组
/// 返回点位分页bz2的md5数组
/// GET /marker_doc/list_page_bin_md5
#[tracing::instrument(skip(auth))]
pub async fn list_page_bin_md5(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let payload = serde_json::json!({});
    match _functions::functions::api::marker_doc::do_list_page_bin_md5(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
