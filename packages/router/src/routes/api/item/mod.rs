mod add;
mod copy;
mod delete;
mod get_by_id;
mod join;
mod list;
mod update;

use anyhow::Result;
use axum::{
    routing::{delete, post, put},
    Router,
};

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/get/list", post(list::get_list))
        .route("/join/{type_id}", post(join::join_type))
        .route("/get/list_by_id", post(get_by_id::get_list_by_id))
        .route("/add", put(add::add))
        .route("/update/{edit_same}", post(update::update))
        .route("/copy/{area_id}", put(copy::copy_to_area))
        .route("/delete/{item_id}", delete(delete::delete));

    Ok(ret)
}