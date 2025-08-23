mod add;
mod delete;
mod get;
mod list;
mod update;

use anyhow::Result;

use axum::{
    routing::{delete, post, put},
    Router,
};

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/get/list", post(list::list))
        .route("/get/:area_id", post(get::get))
        .route("/add", put(add::add))
        .route("/update", post(update::update))
        .route("/:area_id", delete(delete::delete));

    Ok(ret)
}
