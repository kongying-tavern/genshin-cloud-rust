use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;

/// 复制物品到地区
/// 根据物品ID列表复制物品到新地区，此操作会递归复制类型及父级类型。
/// 会返回新的物品列表与新的类型列表，用于反映新的ID
/// PUT /item/copy/{areaId}
#[tracing::instrument(skip(auth))]
pub async fn copy_to_area(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(area_id): Path<i64>,
    Json(payload): Json<Vec<i64>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match crate::functions::api::item::do_copy_to_area(auth, area_id, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(serde_json::json!(v)))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))),
    }
}
