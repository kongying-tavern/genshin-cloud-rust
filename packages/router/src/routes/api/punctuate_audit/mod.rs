mod audit;
mod delete;
mod get;

use anyhow::Result;
use axum::{
    routing::{delete, post},
    Router,
};

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/get/id", post(get::get_id))
        .route("/get/list_byinfo", post(get::get_list_by_info))
        .route("/get/list_byid", post(get::get_list_by_id))
        .route("/get/page/all", post(get::get_page_all))
        .route("/pass/{punctuate_id}", post(audit::pass))
        .route("/reject/{punctuate_id}", post(audit::reject))
        .route("/delete/{punctuate_id}", delete(delete::delete));

    Ok(ret)
}