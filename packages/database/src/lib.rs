pub mod models;

use anyhow::Result;
use lazy_static::lazy_static;
use log::info;
use std::{cell::RefCell, time::Duration};
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

    DB_CONN.lock().await.replace(db);
    info!("Database is registering");
    register().await?;
    info!("Database is ready");

    Ok(())
}

lazy_static! {
    pub static ref DB_CONN: Mutex<RefCell<DatabaseConnection>> = Default::default();
}
