use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;

/// 点位分页数据
/// 查询分页点位信息，返回bz2压缩格式的byte数组
/// GET /marker_doc/list_page_bin/{md5}
#[tracing::instrument(skip(auth))]
pub async fn list_page_bin(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(md5): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::marker_doc::do_list_page_bin(
        auth,
        serde_json::Value::String(md5),
    )
    .await
    {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
