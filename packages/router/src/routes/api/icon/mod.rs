mod add;
mod delete;
mod get_single;
mod list;
mod update;

use anyhow::Result;

use axum::{
    routing::{delete as route_delete, post, put},
    Router,
};

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/get/list", post(list::list))
        .route("/get/single/{iconId}", post(get_single::get_single))
        .route("/update", post(update::update))
        .route("/add", put(add::add))
        .route("/delete/{iconId}", route_delete(delete::delete));

    Ok(ret)
}