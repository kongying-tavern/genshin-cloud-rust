use anyhow::Result;

use axum::{
    Router,
    routing::{get, put},
};

pub(crate) mod get;
pub(crate) mod upload;

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/get", get(get::get))
        .route("/upload/image", put(upload::upload_image));

    Ok(ret)
}
