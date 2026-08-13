use anyhow::Result;

use axum::{extract::Json, http::StatusCode, response::IntoResponse};

use crate::middlewares::{ApiError, AppJson, ExtractAuthInfo};
use _utils::models::Pagination;
use _utils::models::{CommonResponse, ItemAreaPublicListResponse};

/// 列出地区公用物品
/// 列出公共物品，但需要注意处理所属地区已被删除的公共物品
/// POST /item_common/get/list
#[utoipa::path(
    post,
    path = "/api/item_common/get/list",
    tag = "item-common",
    summary = "列出地区公用物品",
    request_body = Pagination,
    responses(
        (status = 200, description = "公用物品分页列表", body = inline(CommonResponse<ItemAreaPublicListResponse>)),
        (status = 401, description = "未登录或令牌无效"),
        (status = 500, description = "服务器内部错误", body = String),
    ),
)]
#[tracing::instrument(skip(auth))]
pub async fn get_list(
    ExtractAuthInfo(auth): ExtractAuthInfo,
    AppJson(payload): AppJson<Pagination>,
) -> Result<impl IntoResponse, ApiError> {
    match crate::functions::api::item_common::do_get_list(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
