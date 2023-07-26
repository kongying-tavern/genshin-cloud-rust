pub mod cache;
pub mod models;

use anyhow::Result;
use lazy_static::lazy_static;
use log::info;
use std::{cell::Cell, time::Duration};
use tokio::sync::Mutex;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};

use models::register;

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

    register(db.clone()).await?;

    info!("Database is ready");
    DB_CONN.lock().await.replace(db);

    Ok(())
}

lazy_static! {
    static ref DB_CONN: Mutex<Cell<DatabaseConnection>> = Default::default();
    static ref DB_CACHE: Mutex<Cell<cache::CacheObject>> = Default::default();
}
