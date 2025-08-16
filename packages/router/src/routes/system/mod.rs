mod action_log;
mod archive;
mod device;
mod invitation;
mod oauth;
mod role;
mod user;

use anyhow::Result;

use axum::Router;

pub async fn router() -> Result<Router> {
    let ret = Router::new();

    Ok(ret)
}
