mod routes;

use std::sync::Arc;

use anyhow::Result;
use axum::Extension;
use hyper::server::Server;
use log::info;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .init();

    let port = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80);

    info!("Site will run on port {}", port);

    Server::bind(&format!("0.0.0.0:{}", port).parse()?)
        .serve(
            routes::register()
                .await?
                .layer(Extension(Arc::new(_functions::SharedDatabaseConnection {
                    conn: _database::init(_database::DatabaseNetworkConfig {
                        host: std::env::var("POSTGRES_HOST").unwrap_or("localhost".into()),
                        port: std::env::var("POSTGRES_PORT")
                            .map(|str| str.parse::<u16>().unwrap())
                            .unwrap_or(5432),
                        username: std::env::var("POSTGRES_USER").unwrap_or("genshin_map".into()),
                        password: std::env::var("POSTGRES_PASSWORD").unwrap_or("root".into()),
                    })
                    .await?,
                    cache: _database::cache::new(),
                })))
                .into_make_service(),
        )
        .await?;

    Ok(())
}
