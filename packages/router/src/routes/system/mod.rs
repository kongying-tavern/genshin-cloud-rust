mod action_log;
mod archive;
mod device;
mod invitation;
pub mod oauth;
mod role;
mod user;

use anyhow::Result;

use axum::{routing::post, Router};

pub async fn router() -> Result<Router> {
    let ret = Router::new();

    Ok(ret)
}
