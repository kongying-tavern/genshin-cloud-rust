use anyhow::Result;

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::middlewares::ExtractAuthInfo;
use _utils::models::item_type::ItemTypeListRequest;

/// 列出某一层级的物品类型
/// 不递归遍历，只遍历子级
/// POST /item_type/get/list/{self}
#[tracing::instrument(skip(db, auth))]
pub async fn get_list(
    State(db): State<crate::routes::SharedDb>,
    ExtractAuthInfo(auth): ExtractAuthInfo,
    Path(self_flag): Path<i64>,
    Json(payload): Json<ItemTypeListRequest>,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match crate::functions::api::item_type::do_get_list(db.as_ref(), auth, self_flag != 0, payload)
        .await
    {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}

/// 列出所有物品类型
/// 不递归遍历，只遍历子级
/// POST /item_type/get/list_all
#[tracing::instrument(skip(db, auth))]
pub async fn get_list_all(
    State(db): State<crate::routes::SharedDb>,
    ExtractAuthInfo(auth): ExtractAuthInfo,
) -> Result<impl IntoResponse, crate::routes::RouteError> {
    match crate::functions::api::item_type::do_get_list_all(db.as_ref(), auth).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err(crate::routes::internal_error(e)),
    }
}
