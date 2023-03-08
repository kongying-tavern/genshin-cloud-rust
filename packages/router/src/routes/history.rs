use std::sync::Arc;

use anyhow::Result;
use axum::{extract::Query, routing::post, Extension, Json, Router};
use serde::{Deserialize, Serialize};

use crate::SharedDatabaseConnection;
use _functions::schemas::history_search::Schema as HistorySearchSchema;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct RollbackQueryParams {
    pub id: Option<i64>,
}

/// 历史记录 API
pub async fn register() -> Result<Router> {
    let router = Router::new()
        .route(
            "/get/list",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<HistorySearchSchema>| async move {
                    // 历史记录分页
                    ""
                },
            ),
        )
        .route(
            "/rollback",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Query(_query): Query<RollbackQueryParams>| async move {
                    // 回滚记录
                    ""
                },
            ),
        );

    Ok(router)
}
