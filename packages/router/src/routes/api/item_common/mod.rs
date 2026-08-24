mod add;
mod delete;
mod list;

use anyhow::Result;
use axum::{
    Router,
    routing::{delete, post, put},
};

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/get/list", post(list::get_list))
        .route("/add", put(add::add))
        .route("/delete/{item_id}", delete(delete::delete));

    Ok(ret)
}
