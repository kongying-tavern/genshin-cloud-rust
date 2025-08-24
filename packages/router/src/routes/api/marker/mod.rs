mod delete;
mod get;
mod single;
mod tweak;

use anyhow::Result;
use axum::{
    routing::{delete, post, put},
    Router,
};

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/get/id", post(get::get_id))
        .route("/get/list_by_info", post(get::get_list_by_info))
        .route("/get/list_by_id", post(get::get_list_by_id))
        .route("/get/page", post(get::get_page))
        .route("/single", put(single::add_single))
        .route("/single", post(single::update_single))
        .route("/{marker_id}", delete(delete::delete))
        .route("/tweak", post(tweak::tweak));

    Ok(ret)
}