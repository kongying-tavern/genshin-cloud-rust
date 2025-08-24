mod add;
mod delete;
mod list;
mod update;

use anyhow::Result;
use axum::{
    routing::{delete, post, put},
    Router,
};

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/get/list", post(list::get_notice_list))
        .route("/add", put(add::add_notice))
        .route("/update", post(update::update_notice))
        .route("/{noticeId}", delete(delete::delete_notice));

    Ok(ret)
}