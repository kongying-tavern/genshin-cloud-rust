mod list;

use anyhow::Result;

use axum::{Router, routing::post};

pub async fn router() -> Result<Router<crate::routes::SharedDb>> {
    let ret = Router::new().route("/get/list", post(list::get_list));

    Ok(ret)
}
