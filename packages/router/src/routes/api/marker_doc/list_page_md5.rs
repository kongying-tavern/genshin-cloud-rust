use _utils::models::CommonResponse;
use anyhow::Result;

use crate::middlewares::{ApiError, ExtractAuthInfo};
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use _functions::functions::api::binary_doc::BinaryMd5Vo;

/// 点位分页的md5数组
/// 返回点位分页bz2的md5数组
/// GET /marker_doc/list_page_bin_md5
#[utoipa::path(
    get,
    path = "/api/marker_doc/list_page_bin_md5",
    tag = "marker-doc",
    summary = "点位分页的 MD5 数组",
    responses(
        (status = 200, description = "MD5 数组", body = inline(CommonResponse<Vec<BinaryMd5Vo>>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn list_page_bin_md5(
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, ApiError> {
    let payload = serde_json::json!({});
    match _functions::functions::api::marker_doc::do_list_page_bin_md5(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
