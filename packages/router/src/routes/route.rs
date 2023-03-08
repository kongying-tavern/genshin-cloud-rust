use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::Path,
    routing::{delete, post, put},
    Extension, Json, Router,
};

use super::PageSearchParams;
use crate::SharedDatabaseConnection;
use _functions::schemas::{
    route::Schema as RouteSchema, route_search::Schema as RouteSearchSchema,
};

/// 路线 API
pub async fn register() -> Result<Router> {
    let router = Router::new()
        .route(
            "/get/page",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<PageSearchParams>| async move {
                    // 分页查询所有路线信息
                    ""
                },
            ),
        )
        .route(
            "/get/search",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<RouteSearchSchema>| async move {
                    // 根据条件筛选分页查询路线信息
                    ""
                },
            ),
        )
        .route(
            "/get/list_byid",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<Vec<i64>>| async move {
                    // 通过 ID 列表批量查询路线信息
                    ""
                },
            ),
        )
        .route(
            "/add",
            put(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<RouteSchema>| async move {
                    // 新增路线，无需指定 ID，ID 由系统自动生成并在响应中返回
                    ""
                },
            ),
        )
        .route(
            "/",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<RouteSchema>| async move {
                    // 修改路线
                    ""
                },
            ),
        )
        .route(
            "/:id",
            delete(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Path(_id): Path<String>| async move {
                    // 删除路线，请在前端做二次确认
                    ""
                },
            ),
        );

    Ok(router)
}
