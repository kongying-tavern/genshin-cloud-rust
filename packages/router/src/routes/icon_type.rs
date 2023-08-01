use anyhow::Result;
use axum::{
    extract::Path,
    routing::{delete, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use _utils::schemas::icon_type::Schema as IconTypeSchema;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct IconTypeSearchParams {
    pub current: Option<i64>,
    pub size: Option<i64>,
    pub typeIdList: Option<Vec<i64>>,
}

/// 图标分类 API
pub async fn register() -> Result<Router> {
    let router = Router::new()
        .route(
            "/get/list",
            post(|Json(_frm): Json<IconTypeSearchParams>| async move {
                // 列出分类
                ""
            }),
        )
        .route(
            "/update",
            post(|Json(_frm): Json<IconTypeSchema>| async move {
                // 修改分类，由类型 ID 来定位修改一个分类
                ""
            }),
        )
        .route(
            "/add",
            put(|Json(_frm): Json<IconTypeSchema>| async move {
                // 新增分类，无需指定 ID，ID 由系统自动生成并在响应中返回
                ""
            }),
        )
        .route(
            "/delete/:id",
            delete(|Path(_id): Path<String>| async move {
                // 删除分类，此操作会递归删除，请在前端做二次确认
                ""
            }),
        );

    Ok(router)
}
