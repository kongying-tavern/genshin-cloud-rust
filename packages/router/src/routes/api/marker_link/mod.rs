mod delete;
mod get;
mod link;

use anyhow::Result;
use axum::{
    routing::{delete, post},
    Router,
};

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/get/list", post(get::get_list))
        .route("/get/graph", post(get::get_graph))
        .route("/link", post(link::link))
        .route("/delete", delete(delete::delete));

    Ok(ret)
}