use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::Path,
    routing::{delete, post},
    Extension, Json, Router,
};

use super::PageSearchParams;
use _functions::schemas::punctuate_search::Schema as PunctuateSearchSchema;
use _functions::SharedDatabaseConnection;

/// 打点 API
pub async fn register() -> Result<Router> {
    let router = Router::new()
        .route(
            "/get/page",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<PunctuateSearchSchema>| async move {
                    // 根据各种条件筛选点位 ID，支持根据末端地区、末端类型和物品类型进行筛选，只能单选
                    ""
                },
            ),
        )
        .route(
            "/get/list_byinfo",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<PunctuateSearchSchema>| async move {
                    // 根据各种条件查询点位信息，支持根据末端地区、末端类型和物品类型进行筛选，只能单选
                    ""
                },
            ),
        )
        .route(
            "/get/list_byid",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<Vec<i64>>| async move {
                    // 通过 ID 列表批量查询点位信息
                    ""
                },
            ),
        )
        .route(
            "/get/page/all",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<PageSearchParams>| async move {
                    // 分页查询所有点位信息
                    ""
                },
            ),
        )
        .route(
            "/pass/:id",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Path(_id): Path<String>| async move {
                    // 通过点位审核，返回点位 ID；通过额外字段进行关联的点位也会自动通过，但不会返回这些点位的 ID
                    ""
                },
            ),
        )
        .route(
            "/reject/:id",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Path(_id): Path<String>,
                 _audit_remark: String| async move {
                    // 驳回点位审核，驳回后该点位和与其以额外字段关联的点位都会回到暂存区
                    ""
                },
            ),
        )
        .route(
            "/delete/:id",
            delete(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Path(_id): Path<String>| async move {
                    // 删除审核点位，根据 ID 删除，同时删除与其以额外字段关联的点位
                    ""
                },
            ),
        );

    Ok(router)
}
