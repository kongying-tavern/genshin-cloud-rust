use anyhow::Result;
use axum::{
    extract::Path,
    routing::{delete, post, put},
    Json, Router,
};

use super::PageSearchParams;

use _utils::schemas::{
    marker::Schema as MarkerSchema, marker_search::Schema as MarkerSearchSchema,
};

/// 点位 API
pub async fn register() -> Result<Router> {
    let router = Router::new()
        .route(
            "/get/id",
            post(|Json(_frm): Json<MarkerSearchSchema>| async move {
                // 根据各种条件筛选点位 ID，支持根据末端地区、末端类型和物品类型进行筛选，只能单选
                ""
            }),
        )
        .route(
            "/get/list_byinfo",
            post(|Json(_frm): Json<MarkerSearchSchema>| async move {
                // 根据各种条件查询点位信息，支持根据末端地区、末端类型和物品类型进行筛选，只能单选
                ""
            }),
        )
        .route(
            "/get/list_byid",
            post(|Json(_frm): Json<Vec<i64>>| async move {
                // 通过 ID 列表批量查询点位信息
                ""
            }),
        )
        .route(
            "/get/page",
            post(|Json(_frm): Json<PageSearchParams>| async move {
                // 分页查询所有点位信息
                ""
            }),
        )
        .route(
            "/single",
            post(|Json(_frm): Json<MarkerSchema>| async move {
                // 修改点位
                ""
            }),
        )
        .route(
            "/single",
            put(|Json(_frm): Json<MarkerSchema>| async move {
                // 新增点位，无需指定 ID，ID 由系统自动生成并在响应中返回
                ""
            }),
        )
        .route(
            "/:id",
            delete(|Path(_id): Path<String>| async move {
                // 删除点位
                ""
            }),
        );

    Ok(router)
}
