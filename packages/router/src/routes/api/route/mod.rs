mod delete;
mod get;
mod manage;

use anyhow::Result;

use axum::{
    routing::{delete, post, put},
    Router,
};

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/get/page", post(get::get_page))
        .route("/get/search", post(get::get_search))
        .route("/get/list_by_id", post(get::get_list_by_id))
        .route("/add", put(manage::add))
        .route("/", post(manage::update))
        .route("/{route_id}", delete(delete::delete));

    Ok(ret)
}
