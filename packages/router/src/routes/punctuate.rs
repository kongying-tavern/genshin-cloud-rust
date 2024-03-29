use anyhow::Result;
use axum::{
    extract::Path,
    routing::{delete, post},
    Json, Router,
};

use super::PageSearchParams;

use _utils::schemas::punctuate_search::Schema as PunctuateSearchSchema;

/// 打点 API
pub async fn register() -> Result<Router> {
    let router = Router::new()
        .route(
            "/get/page",
            post(|Json(_frm): Json<PunctuateSearchSchema>| async move {
                // 根据各种条件筛选点位 ID，支持根据末端地区、末端类型和物品类型进行筛选，只能单选
                ""
            }),
        )
        .route(
            "/get/list_byinfo",
            post(|Json(_frm): Json<PunctuateSearchSchema>| async move {
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
            "/get/page/all",
            post(|Json(_frm): Json<PageSearchParams>| async move {
                // 分页查询所有点位信息
                ""
            }),
        )
        .route(
            "/pass/:id",
            post(|Path(_id): Path<String>| async move {
                // 通过点位审核，返回点位 ID；通过额外字段进行关联的点位也会自动通过，但不会返回这些点位的 ID
                ""
            }),
        )
        .route(
            "/reject/:id",
            post(
                |Path(_id): Path<String>, _audit_remark: String| async move {
                    // 驳回点位审核，驳回后该点位和与其以额外字段关联的点位都会回到暂存区
                    ""
                },
            ),
        )
        .route(
            "/delete/:id",
            delete(|Path(_id): Path<String>| async move {
                // 删除审核点位，根据 ID 删除，同时删除与其以额外字段关联的点位
                ""
            }),
        );

    Ok(router)
}
