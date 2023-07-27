use std::sync::Arc;

use anyhow::Result;
use axum::{routing::post, Extension, Json, Router};

use _functions::SharedDatabaseConnection;
use _utils::schemas::score_stat::Schema as ScoreStatSchema;

/// 评分统计 API
pub async fn register() -> Result<Router> {
    let router = Router::new()
        .route(
            "/generate",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<ScoreStatSchema>| async move {
                    // 生成评分数据
                    ""
                },
            ),
        )
        .route(
            "/data",
            post(
                |Extension(_db): Extension<Arc<SharedDatabaseConnection>>,
                 Json(_frm): Json<ScoreStatSchema>| async move {
                    // 获取评分数据
                    ""
                },
            ),
        );

    Ok(router)
}
