use anyhow::Result;
use axum::{
    extract::Path,
    routing::{delete, post, put},
    Json, Router,
};

use super::PageSearchParams;

/// 公用物品 API
pub async fn register() -> Result<Router> {
    let router = Router::new()
        .route(
            "/get/list",
            post(|Json(_frm): Json<PageSearchParams>| async move {
                // 列出地区公用物品
                ""
            }),
        )
        .route(
            "/add",
            put(|Json(_frm): Json<Vec<i64>>| async move {
                // 新增地区公用物品，通过 ID 列表批量增加
                ""
            }),
        )
        .route(
            "/delete/:id",
            delete(|Path(_id): Path<String>| async move {
                // 删除地区公用物品
                ""
            }),
        );

    Ok(router)
}
