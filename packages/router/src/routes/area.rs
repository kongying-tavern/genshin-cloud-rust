use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::Path,
    routing::{delete, post, put},
    Extension, Json, Router,
};

use _functions::{
    functions::{RequestData, DEFAULT_ERROR_JSON_MSG},
    SharedDatabaseConnection,
};
use _utils::schemas::{area::Schema as AreaSchema, area_search::Schema as AreaSearchSchema};

/// 地区 API
pub async fn register() -> Result<Router> {
    let router = Router::new()
        .route(
            "/get/list",
            post(
                |Extension(db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<AreaSearchSchema>| async move {
                    // 列出地区，可根据父级地区id列出子地区列表
                    serde_json::to_string(&RequestData::new(
                        _functions::functions::area::list_area(&db).await,
                    ))
                    .unwrap_or(DEFAULT_ERROR_JSON_MSG.into())
                },
            ),
        )
        .route(
            "/get/:id",
            post(
                |Extension(db): Extension<Arc<SharedDatabaseConnection>>,
                 Path(id): Path<String>| async move {
                    // 获取单个地区信息
                    match id.parse::<i64>() {
                        Ok(id) => {
                            let area = _functions::functions::area::get_area(&db, id).await;
                            serde_json::to_string(&RequestData::new(area))
                                .unwrap_or(DEFAULT_ERROR_JSON_MSG.into())
                        }
                        Err(_) => serde_json::to_string(&RequestData::<()>::new(Err(
                            anyhow::anyhow!("The parameter is not a number"),
                        )))
                        .unwrap_or(DEFAULT_ERROR_JSON_MSG.into()),
                    }
                },
            ),
        )
        .route(
            "/add",
            put(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(frm): Json<AreaSchema>| async move {
                    // 新增地区，返回新增地区ID
                    format!("{:?}", frm)
                },
            ),
        )
        .route(
            "/",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(frm): Json<AreaSchema>| async move {
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
