mod list;

use anyhow::Result;

use axum::{routing::post, Router};

pub async fn router() -> Result<Router> {
    let ret = Router::new().route("/get/list", post(list::get_list));

    Ok(ret)
}
