use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::Path,
    routing::{delete, post, put},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};

use _functions::schemas::tag_type::Schema as TagTypeSchema;
use _functions::SharedDatabaseConnection;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct TagTypeSearchParams {
    // 当前页，从 0 开始
    pub current: Option<i64>,
    // 每页大小，默认为 10
    pub size: Option<i64>,
    // 父级类型 ID 列表
    pub typeIdList: Option<Vec<i64>>,
}

/// 图标标签分类 API
pub async fn register() -> Result<Router> {
    let router = Router::new()
        .route(
            "/get/list",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<TagTypeSearchParams>| async move {
                    // 列出标签分类；parent_id 为 -1 时列出所有根分类，isTraverse 为 1 时遍历所有子分类
                    ""
                },
            ),
        )
        .route(
            "/add",
            put(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<TagTypeSchema>| async move {
                    // 新增分类
                    ""
                },
            ),
        )
        .route(
            "/update",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<TagTypeSchema>| async move {
                    // 修改分类
                    ""
                },
            ),
        )
        .route(
            "/delete/:id",
            delete(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Path(_id): Path<String>| async move {
                    // 删除分类；此操作会递归删除，请在前端做二次确认
                    ""
                },
            ),
        );

    Ok(router)
}
