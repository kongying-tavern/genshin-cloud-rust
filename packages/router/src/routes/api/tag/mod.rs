mod add;
mod delete;
mod list;
mod update;
mod update_type;

use anyhow::Result;

use axum::{
    Router,
    routing::{delete as route_delete, post, put},
};

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/get/list", post(list::list))
        .route("/add", put(add::add))
        .route("/update", post(update::update))
        .route("/updateType", post(update_type::update_type))
        .route("/delete/{tagId}", route_delete(delete::delete));

    Ok(ret)
}
