use anyhow::Result;

use axum::{Router, routing::get};

mod all_bin;
mod all_bin_md5;

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/all_bin_md5", get(all_bin_md5::all_bin_md5))
        .route("/all_bin", get(all_bin::all_bin));

    Ok(ret)
}
