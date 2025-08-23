mod add;
mod delete;
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
        .route("/add", put(add::add))
        .route("/update", post(update::update))
        .route("/delete/{typeId}", route_delete(delete::delete));

    Ok(ret)
}