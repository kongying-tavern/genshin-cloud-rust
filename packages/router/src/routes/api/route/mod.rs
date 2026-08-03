mod delete;
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
        .route("/get/search", post(get::get_search))
        .route("/get/list_byid", post(get::get_list_by_id))
        .route("/add", put(manage::add))
        .route("/", post(manage::update))
        .route("/{route_id}", delete(delete::delete))
        // 前端契约路径无尾斜杠（`/api/route`）；axum nest 精确匹配无斜杠
        // 形态，尾斜杠请求（`/route/`）落到 fallback——交给同一个 handler。
        .fallback(manage::update);

    Ok(ret)
}
