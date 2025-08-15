pub mod models;

use anyhow::{anyhow, Result};
use log::info;
use once_cell::sync::Lazy;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use sea_orm::{ConnectOptions, Database, DatabaseConnection};

pub static DB_CONN: Lazy<Arc<Mutex<Option<DatabaseConnection>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

pub struct DatabaseNetworkConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

pub async fn init(config: DatabaseNetworkConfig) -> Result<()> {
    let mut opt = ConnectOptions::new(format!(
        "postgres://{}:{}@{}:{}/{}",
        config.username, config.password, config.host, config.port, config.database
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

    DB_CONN
        .lock()
        .map_err(|err| anyhow!(format!("Failed to lock DB_CONN: {}", err)))?
        .replace(db);
    info!("Database is ready");

    Ok(())
}
