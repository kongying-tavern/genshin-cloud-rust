mod add;
mod delete;
mod list;
mod move_type;
mod update;

use anyhow::Result;

use axum::{
    routing::{delete, post, put},
    Router,
};

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/get/list/{self}", post(list::get_list))
        .route("/get/list_all", post(list::get_list_all))
        .route("/add", put(add::add))
        .route("/update", post(update::update))
        .route("/move/{target_type_id}", post(move_type::move_to_target))
        .route("/delete/{item_type_id}", delete(delete::delete));

    Ok(ret)
}
