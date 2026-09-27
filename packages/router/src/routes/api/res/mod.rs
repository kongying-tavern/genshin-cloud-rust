use anyhow::Result;

use axum::{
    Router,
    routing::{get, put},
};

mod get;
mod upload;

pub async fn router() -> Result<Router<crate::routes::SharedDb>> {
    let ret = Router::new()
        .route("/get", get(get::get))
        .route("/upload/image", put(upload::upload_image));

    Ok(ret)
}
