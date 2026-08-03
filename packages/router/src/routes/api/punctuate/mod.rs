mod actions;
mod get;
mod manage;

use anyhow::Result;
use axum::{
    Router,
    routing::{delete, post, put},
};

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/get/page", post(get::get_page))
        .route("/get/page/{author_id}", post(get::get_page_by_author))
        .route("/", post(manage::update))
        .route("/", put(manage::submit))
        .route("/push/{author_id}", put(actions::push))
        .route(
            "/delete/{author_id}/{punctuate_id}",
            delete(actions::delete),
        )
        // 前端契约路径带尾斜杠（`/api/punctuate/`）；axum 的 nest 精确匹配
        // 无斜杠形态，尾斜杠请求落到 fallback——交给同一个 update handler。
        .fallback(manage::update);

    Ok(ret)
}
