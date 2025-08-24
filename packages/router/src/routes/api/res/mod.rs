mod upload;

use anyhow::Result;

use axum::{routing::put, Router};

pub async fn router() -> Result<Router> {
    let ret = Router::new().route("/upload/image", put(upload::upload_image));

    Ok(ret)
}
