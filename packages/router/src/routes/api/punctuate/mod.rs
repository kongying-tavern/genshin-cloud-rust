mod actions;
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
        .route("/get/page/{author_id}", post(get::get_page_by_author))
        .route("/", post(manage::update))
        .route("/", put(manage::submit))
        .route("/push/{author_id}", put(actions::push))
        .route("/delete/{author_id}/{punctuate_id}", delete(actions::delete));

    Ok(ret)
}
