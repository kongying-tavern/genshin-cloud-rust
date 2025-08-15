mod routes;

use anyhow::Result;
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

    _database::init(_database::DatabaseNetworkConfig {
        host: std::env::var("DB_HOST").unwrap_or("localhost".into()),
        port: std::env::var("DB_PORT")
            .map(|str| str.parse::<u16>().unwrap())
            .unwrap_or(5432),
        username: std::env::var("DB_USERNAME").unwrap_or("genshin_map".into()),
        password: std::env::var("DB_PASSWORD").unwrap_or("root".into()),
        database: std::env::var("DB_DATABASE").unwrap_or("backend".into()),
    })
    .await?;

    Server::bind(&format!("0.0.0.0:{}", port).parse()?)
        .serve(routes::register().await?.into_make_service())
        .await?;

    Ok(())
}
