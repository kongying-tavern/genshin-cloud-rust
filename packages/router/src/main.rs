pub mod functions;
mod middlewares;
mod routes;

use anyhow::Result;
use std::net::SocketAddr;

use axum::serve;
use tokio::net::TcpListener;

use crate::routes::router;
use _database::init_db_conn;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    // Install the ring crypto provider for jsonwebtoken (v10 requires an
    // explicit process-level CryptoProvider). This must happen before any
    // JWT encode/decode call. We use ring (not aws-lc-rs) to stay consistent
    // with the workspace's aws-lc-free rustls+ring policy.
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .init();

    let port = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80);

    log::info!("Site will run on port {}", port);
    init_db_conn().await?;

    let router = router()
        .await?
        .into_make_service_with_connect_info::<SocketAddr>();

    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("Failed to bind");
    serve(listener, router).await?;

    Ok(())
}
