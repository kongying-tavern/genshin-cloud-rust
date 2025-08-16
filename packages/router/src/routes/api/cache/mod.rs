use anyhow::Result;

use axum::Router;

pub async fn router() -> Result<Router> {
    let ret = Router::new();

    Ok(ret)
}
