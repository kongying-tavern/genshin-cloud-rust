use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::Path,
    routing::{delete, post, put},
    Extension, Json, Router,
};

use _functions::schemas::{icon::Schema as IconSchema, icon_search::Schema as IconSearchSchema};
use _functions::SharedDatabaseConnection;

/// 图标 API
pub async fn register() -> Result<Router> {
    let router = Router::new()
        .route(
            "/get/list",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<IconSearchSchema>| async move {
                    // 列出图标，可按照分类和上传者进行查询，也可根据ID批量分页查询
                    ""
                },
            ),
        )
        .route(
            "/get/single/:id",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Path(_id): Path<String>| async move {
                    // 获取单个图标信息
                    ""
                },
            ),
        )
        .route(
            "/update",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<IconSchema>| async move {
                    // 修改图标信息，由 icon_id 定位修改
                    ""
                },
            ),
        )
        .route(
            "/add",
            put(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<IconSchema>| async move {
                    // 新增图标，无需指定 ID，ID 由系统自动生成并在响应中返回
                    ""
                },
            ),
        )
        .route(
            "/delete/:id",
            delete(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Path(_id): Path<String>| async move {
                    // 删除图标
                    ""
                },
            ),
        );

    Ok(router)
}
