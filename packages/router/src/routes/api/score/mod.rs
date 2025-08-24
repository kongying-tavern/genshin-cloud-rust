mod data;
mod generate;

use anyhow::Result;
use axum::{
    routing::post,
    Router,
};

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/generate", post(generate::generate_score))
        .route("/data", post(data::get_score_data));

    Ok(ret)
}