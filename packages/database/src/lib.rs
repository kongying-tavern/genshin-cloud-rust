pub mod models;

use anyhow::{anyhow, Result};
use log::info;
use once_cell::sync::Lazy;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use sea_orm::{ConnectOptions, Database, DatabaseConnection};

pub static PG_CONN: Lazy<Arc<Mutex<Option<DatabaseConnection>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));
pub static REDIS_CONN: Lazy<Arc<Mutex<Option<redis::Client>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));
pub static MINIO_CONN: Lazy<Arc<Mutex<Option<minio::s3::Client>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

pub async fn init() -> Result<()> {
    PG_CONN
        .lock()
        .map_err(|err| anyhow!(format!("Failed to lock PG_CONN: {}", err)))?
        .replace(
            Database::connect({
                let mut opt = ConnectOptions::new(format!(
                    "postgres://{}:{}@{}:{}/{}",
                    std::env::var("DB_HOST").unwrap_or("localhost".into()),
                    std::env::var("DB_PORT")
                        .map(|str| str.parse::<u16>().unwrap())
                        .unwrap_or(5432),
                    std::env::var("DB_USERNAME").unwrap_or("genshin_map".into()),
                    std::env::var("DB_PASSWORD").unwrap_or("".into()),
                    std::env::var("DB_DATABASE").unwrap_or("genshin_map".into()),
                ));
                opt.max_connections(100)
                    .min_connections(5)
                    .connect_timeout(Duration::from_secs(8))
                    .acquire_timeout(Duration::from_secs(8))
                    .idle_timeout(Duration::from_secs(8))
                    .max_lifetime(Duration::from_secs(8))
                    .sqlx_logging(true)
                    .sqlx_logging_level(log::LevelFilter::Trace);
                opt
            })
            .await?,
        );
    info!("Postgres is ready");

    REDIS_CONN
        .lock()
        .map_err(|err| anyhow!(format!("Failed to lock REDIS_CONN: {}", err)))?
        .replace(redis::Client::open(format!(
            "redis://{}{}@{}:{}/{}",
            std::env::var("REDIS_USERNAME").unwrap_or("".into()),
            std::env::var("REDIS_PASSWORD")
                .map(|p| format!(":{}", p))
                .unwrap_or_default(),
            std::env::var("REDIS_HOST").unwrap_or("localhost".into()),
            std::env::var("REDIS_PORT")
                .map(|str| str.parse::<u16>().unwrap())
                .unwrap_or(6379),
            std::env::var("REDIS_DATABASE").unwrap_or("genshin_map".into()),
        ))?);
    info!("Redis is ready");

    MINIO_CONN
        .lock()
        .map_err(|err| anyhow!(format!("Failed to lock MINIO_CONN: {}", err)))?
        .replace(minio::s3::Client::new(
            std::env::var("MINIO_BASE_URL")
                .unwrap_or("localhost".into())
                .parse()?,
            Some(Box::new(minio::s3::creds::StaticProvider::new(
                &std::env::var("MINIO_KEY").unwrap_or("".into()),
                &std::env::var("MINIO_SECRET").unwrap_or("".into()),
                None,
            ))),
            None,
            None,
        )?);
    info!("MinIO is ready");

    Ok(())
}
