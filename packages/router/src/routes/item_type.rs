use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::Path,
    routing::{delete, post, put},
    Extension, Json, Router,
};

use crate::SharedDatabaseConnection;
use _functions::schemas::item_type::Schema as ItemTypeSchema;

/// 物品分类 API
pub async fn register() -> Result<Router> {
    let router = Router::new()
        .route(
            "/get/list/:is_search_self",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Path(_is_search_self): Path<String>| async move {
                    // 列出物品类型，不递归遍历，只遍历直接子级
                    // is_search_self 的值为 0 时查询自身，为 1 时查询子级
                    ""
                },
            ),
        )
        .route(
            "/get/list_all",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>| async move {
                    // 列出所有物品类型，返回所有可访问的物品类型
                    ""
                },
            ),
        )
        .route(
            "/update",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<ItemTypeSchema>| async move {
                    // 修改物品类型
                    ""
                },
            ),
        )
        .route(
            "/add",
            put(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<ItemTypeSchema>| async move {
                    // 添加物品类型，无需指定 ID，ID 由系统自动生成并在响应中返回
                    ""
                },
            ),
        )
        .route(
            "/move/:id",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Path(_id): Path<String>,
                 Json(_frm): Json<Vec<i64>>| async move {
                    // 批量移动物品类型为另一个物品类型的子类型
                    ""
                },
            ),
        )
        .route(
            "/delete/:id",
            delete(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Path(_id): Path<String>| async move {
                    // 删除物品类型，此操作会递归删除，请在前端做二次确认
                    ""
                },
            ),
        );

    Ok(router)
}
