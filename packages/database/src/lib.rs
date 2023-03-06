pub mod models;

use log::info;
use std::time::Duration;

use models::register;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

pub struct DatabaseNetworkConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

pub async fn init(
    config: DatabaseNetworkConfig,
) -> Result<Box<DatabaseConnection>, Box<dyn std::error::Error>> {
    let mut opt = ConnectOptions::new(format!(
        "postgres://{}:{}@{}:{}/kongyin",
        config.username, config.password, config.host, config.port
    ));
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Trace);
    let db = Database::connect(opt).await?;

    register(db.clone()).await?;

    info!("Database is ready");
    Ok(Box::new(db))
}
