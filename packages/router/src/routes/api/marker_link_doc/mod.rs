mod doc;

use anyhow::Result;

use axum::{Router, routing::get};

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/all_list_bin_md5", get(doc::all_bin_md5))
        .route("/all_list_bin", get(doc::all_bin))
        .route("/all_graph_bin_md5", get(doc::all_graph_bin_md5))
        .route("/all_graph_bin", get(doc::all_graph_bin));

    Ok(ret)
}
