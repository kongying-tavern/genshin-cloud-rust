use std::sync::Arc;

use axum::{
    extract::Path,
    routing::{delete, post, put},
    Extension, Form, Json, Router,
};
use log::info;
use serde::{Deserialize, Serialize};

use crate::SharedDatabaseConnection;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AreaDto {
    pub name: String,
    pub code: Option<String>,
    pub content: Option<String>,
    pub icon_tag: String,
    pub parent_id: i64,
    pub is_final: bool,
    pub hidden_flag: i32,
    pub sort_index: i32,
    pub sync_tag: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct AreaGetForm {
    pub userDataLevel: Option<String>,
}

// 地区 API
pub async fn register() -> Result<Router, Box<dyn std::error::Error>> {
    let router = Router::new()
        .route(
            "/get/list",
            post(
                |Extension(db): Extension<Arc<SharedDatabaseConnection>>,
                 Form(_params): Form<AreaGetForm>| async move {
                    // 列出地区，可根据父级地区id列出子地区列表
                    _functions::functions::area::list_area(db.conn.clone())
                        .await
                        .unwrap_or("Error".into())
                },
            ),
        )
        .route(
            "/get/:id",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Path(id): Path<String>,
                 Form(params): Form<AreaGetForm>| async move {
                    // 获取单个地区信息
                    info!("id: {}", id);
                    info!("form: {:?}", params);
                    id
                },
            ),
        )
        .route(
            "/add",
            put(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(frm): Json<AreaDto>| async move {
                    // 新增地区，返回新增地区ID
                    format!("{:?}", frm)
                },
            ),
        )
        .route(
            "/",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(frm): Json<AreaDto>| async move {
                    // 修改地区
                    format!("{:?}", frm)
                },
            ),
        )
        .route(
            "/:id",
            delete(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Path(id): Path<String>| async move {
                    // 删除地区；此操作会递归删除，请在前端做二次确认
                    id
                },
            ),
        );

    Ok(router)
}
