use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::CommonResponse;
use anyhow::Result;

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};

/// 将物品加入某一类型
/// 根据物品ID列表批量加入
/// POST /item/join/{typeId}
#[utoipa::path(
    post,
    path = "/api/item/join/{type_id}",
    tag = "item",
    summary = "将物品批量加入某一类型",
    params(("type_id" = i64, Path, description = "物品类型 ID")),
    request_body = Vec<i64>,
    responses(
        (status = 200, description = "加入结果", body = inline(CommonResponse<utoipa::TupleUnit>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn join_type(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(type_id): Path<i64>,
    AppJson(payload): AppJson<Vec<i64>>,
) -> Result<impl IntoResponse, ApiError> {
    match _functions::functions::api::item::do_join_type(auth, type_id, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(serde_json::json!(v)))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
