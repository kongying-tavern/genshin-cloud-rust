use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::Path,
    routing::{delete, post, put},
    Extension, Json, Router,
};

use super::PageSearchParams;
use _functions::schemas::marker_punctuate::Schema as PunctuateSchema;
use _functions::SharedDatabaseConnection;

/// 打点审核 API
pub async fn register() -> Result<Router> {
    let router = Router::new()
        .route(
            "/get/page",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<PageSearchParams>| async move {
                    // 分页查询所有的个人打点信息
                    ""
                },
            ),
        )
        .route(
            "/get/page/:author_id",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Path(_id): Path<String>,
                 Json(_frm): Json<PageSearchParams>| async move {
                    // 分页查询个人未通过的打点信息，注意无法越权查看他人的
                    ""
                },
            ),
        )
        .route(
            "/",
            put(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<PunctuateSchema>| async move {
                    // 提交暂存点位，成功则返回打点提交 ID
                    ""
                },
            ),
        )
        .route(
            "/push/:author_id",
            put(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Path(_author_id): Path<String>| async move {
                    // 将个人暂存点位全部提交，进入审核
                    ""
                },
            ),
        )
        .route(
            "/",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<PunctuateSchema>| async move {
                    // 修改个人尚未提交的点位，修改后的点位仍然是暂存状态
                    ""
                },
            ),
        )
        .route(
            "/delete/:author_id/:punctuate_id",
            delete(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Path((_author_id, _punctuate_id)): Path<(String, String)>| async move {
                    // 删除个人的某个暂存点位
                    ""
                },
            ),
        );

    Ok(router)
}
