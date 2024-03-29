use anyhow::Result;
use axum::{
    extract::Path,
    routing::{delete, post, put},
    Json, Router,
};

use _utils::schemas::{icon::Schema as IconSchema, icon_search::Schema as IconSearchSchema};

/// 图标 API
pub async fn register() -> Result<Router> {
    let router = Router::new()
        .route(
            "/get/list",
            post(|Json(_frm): Json<IconSearchSchema>| async move {
                // 列出图标，可按照分类和上传者进行查询，也可根据ID批量分页查询
                ""
            }),
        )
        .route(
            "/get/single/:id",
            post(|Path(_id): Path<String>| async move {
                // 获取单个图标信息
                ""
            }),
        )
        .route(
            "/update",
            post(|Json(_frm): Json<IconSchema>| async move {
                // 修改图标信息，由 icon_id 定位修改
                ""
            }),
        )
        .route(
            "/add",
            put(|Json(_frm): Json<IconSchema>| async move {
                // 新增图标，无需指定 ID，ID 由系统自动生成并在响应中返回
                ""
            }),
        )
        .route(
            "/delete/:id",
            delete(|Path(_id): Path<String>| async move {
                // 删除图标
                ""
            }),
        );

    Ok(router)
}
