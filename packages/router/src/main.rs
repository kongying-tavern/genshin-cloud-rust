mod middlewares;
mod routes;

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

    _database::init(_database::DatabaseNetworkConfig {
        host: std::env::var("DB_HOST").unwrap_or("localhost".into()),
        port: std::env::var("DB_PORT")
            .map(|str| str.parse::<u16>().unwrap())
            .unwrap_or(5432),
        username: std::env::var("DB_USERNAME").unwrap_or("genshin_map".into()),
        password: std::env::var("DB_PASSWORD").unwrap_or("".into()),
        database: std::env::var("DB_DATABASE").unwrap_or("genshin_map".into()),
    })
    .await?;

    let router = router()
        .await?
        .into_make_service_with_connect_info::<SocketAddr>();

    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("Failed to bind");
    serve(listener, router).await?;

    Ok(())
}
