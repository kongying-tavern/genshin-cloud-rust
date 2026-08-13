use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::CommonResponse;
use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

/// 复制物品到地区
/// 根据物品ID列表复制物品到新地区，此操作会递归复制类型及父级类型。
/// 会返回新的物品列表与新的类型列表，用于反映新的ID
/// PUT /item/copy/{areaId}
#[utoipa::path(
    put,
    path = "/api/item/copy/{area_id}",
    tag = "item",
    summary = "复制物品到地区",
    params(("area_id" = i64, Path, description = "目标地区 ID")),
    request_body = Vec<i64>,
    responses(
        (status = 200, description = "新物品/新类型 ID 列表", body = inline(CommonResponse<Vec<i64>>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn copy_to_area(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(area_id): Path<i64>,
    AppJson(payload): AppJson<Vec<i64>>,
) -> Result<impl IntoResponse, ApiError> {
    match crate::functions::api::item::do_copy_to_area(auth, area_id, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(serde_json::json!(v)))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
