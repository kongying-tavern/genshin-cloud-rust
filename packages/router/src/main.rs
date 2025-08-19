mod middlewares;
mod routes;

use _database::DB_CONN;
use anyhow::Result;
use std::net::SocketAddr;

use axum::serve;
use tokio::net::TcpListener;

use crate::routes::router;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .init();

    let port = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80);

    log::info!("Site will run on port {}", port);

    DB_CONN.pg_conn.ping().await?;

    let router = router()
        .await?
        .into_make_service_with_connect_info::<SocketAddr>();

    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("Failed to bind");
    serve(listener, router).await?;

    Ok(())
}
