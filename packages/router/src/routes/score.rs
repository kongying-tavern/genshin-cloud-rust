use anyhow::Result;
use axum::{routing::post, Json, Router};

use _utils::schemas::score_stat::Schema as ScoreStatSchema;

/// 评分统计 API
pub async fn register() -> Result<Router> {
    let router = Router::new()
        .route(
            "/generate",
            post(|Json(_frm): Json<ScoreStatSchema>| async move {
                // 生成评分数据
                ""
            }),
        )
        .route(
            "/data",
            post(|Json(_frm): Json<ScoreStatSchema>| async move {
                // 获取评分数据
                ""
            }),
        );

    Ok(router)
}
