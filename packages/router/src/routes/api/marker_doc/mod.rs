mod list_page_bin;
mod list_page_md5;

use anyhow::Result;
use axum::{
    routing::get,
    Router,
};

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/list_page_bin_md5", get(list_page_md5::list_page_bin_md5))
        .route("/list_page_bin/{md5}", get(list_page_bin::list_page_bin));

    Ok(ret)
}