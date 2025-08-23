mod create;
mod delete;
mod get_single;
mod list;
mod update_association;
mod update_type;

use anyhow::Result;

use axum::{
    routing::{delete as route_delete, post, put},
    Router,
};

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/get/list", post(list::list))
        .route("/get/single/{name}", post(get_single::get_single))
        .route(
            "/{tag_name}/{icon_id}",
            post(update_association::update_association),
        )
        .route("/update_type", post(update_type::update_type))
        .route("/{tag_name}", put(create::create))
        .route("/{tag_name}", route_delete(delete::delete));

    Ok(ret)
}
