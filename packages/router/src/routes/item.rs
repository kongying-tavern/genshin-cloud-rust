use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::Path,
    routing::{delete, post, put},
    Extension, Json, Router,
};

use crate::SharedDatabaseConnection;
use _functions::schemas::item::Schema as ItemSchema;

/// 物品 API
pub async fn register() -> Result<Router> {
    let router = Router::new()
        .route(
            "/get/list_byid",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<Vec<i64>>| async move {
                    // 根据物品 ID 查询物品
                    ""
                },
            ),
        )
        .route(
            "/get/list",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>| async move {
                    // 根据筛选条件列出物品信息
                    ""
                },
            ),
        )
        .route(
            "/update/:is_edit_same",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Path(_is_edit_same): Path<String>,
                 Json(_frm): Json<ItemSchema>| async move {
                    // 修改物品
                    // is_edit_same 的值为 0 时只修改单个物品，为 1 时同时修改同类物品信息
                    ""
                },
            ),
        )
        .route(
            "/join/:type_id",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Path(_type_id): Path<String>,
                 Json(_frm): Json<Vec<i64>>| async move {
                    // 将批量物品加入某一类型；在加入多类型时需要注意类型的地区需一致
                    ""
                },
            ),
        )
        .route(
            "/add",
            put(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<ItemSchema>| async move {
                    // 添加物品，无需指定 ID，ID 由系统自动生成并在响应中返回
                    ""
                },
            ),
        )
        .route(
            "/copy/:area_id",
            put(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Path(_area_id): Path<String>,
                 Json(_frm): Json<Vec<i64>>| async move {
                    // 复制物品到地区，根据物品 ID 复制物品到指定地区，并且会递归复制父类型，返回新的物品 ID 列表
                    ""
                },
            ),
        )
        .route(
            "/delete/:id",
            delete(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Path(_id): Path<String>| async move {
                    // 删除物品
                    ""
                },
            ),
        );

    Ok(router)
}
