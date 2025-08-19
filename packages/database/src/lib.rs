pub mod models;

use anyhow::{anyhow, Context, Result};
use log::info;
use minio::s3::types::S3Api;
use std::{sync::Arc, time::Duration};

use sea_orm::{ConnectOptions, Database, DatabaseConnection};

#[derive(Debug, Clone)]
pub struct DatabaseConnectionMap {
    pub pg_conn: DatabaseConnection,
    pub redis_conn: redis::Client,
    pub minio_conn: minio::s3::Client,
}

use once_cell::sync::OnceCell;

pub static DB_CONN: OnceCell<Arc<DatabaseConnectionMap>> = OnceCell::new();

pub async fn init_db_conn() -> anyhow::Result<()> {
    let conn_map = Arc::new(build_db_map().await?);
    DB_CONN
        .set(conn_map)
        .map_err(|_| anyhow!("DB_CONN already initialized"))
}

async fn build_db_map() -> Result<DatabaseConnectionMap> {
    // Postgres
    let pg_conn = {
        let mut opt = ConnectOptions::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            std::env::var("DB_USERNAME").unwrap_or("genshin_map".into()),
            std::env::var("DB_PASSWORD").unwrap_or("".into()),
            std::env::var("DB_HOST").unwrap_or("localhost".into()),
            std::env::var("DB_PORT")
                .map(|str| str.parse::<u16>().unwrap())
                .unwrap_or(5432),
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
        Database::connect(opt).await?
    };
    info!("Postgres is ready");

    // Redis
    let redis_conn = redis::Client::open(format!(
        "redis://{}{}@{}:{}/{}",
        std::env::var("REDIS_USERNAME").unwrap_or("".into()),
        std::env::var("REDIS_PASSWORD")
            .map(|p| format!(":{}", p))
            .unwrap_or_default(),
        std::env::var("REDIS_HOST").unwrap_or("localhost".into()),
        std::env::var("REDIS_PORT")
            .map(|str| str.parse::<u16>().unwrap())
            .unwrap_or(6379),
        1,
    ))?;
    info!("Redis is ready");

    // MinIO
    let minio_conn = {
        let client = minio::s3::Client::new(
            std::env::var("MINIO_BASE_URL")
                .unwrap_or("http://localhost:9000".into())
                .parse()?,
            Some(Box::new(minio::s3::creds::StaticProvider::new(
                &std::env::var("MINIO_ACCESS_KEY").context("MINIO_ACCESS_KEY must be set")?,
                &std::env::var("MINIO_SECRET_KEY").context("MINIO_SECRET_KEY must be set")?,
                None,
            ))),
            None,
            None,
        )?;

        // Ensure buckets exist and set policy
        for bucket in ["images", "bz2doc"] {
            if !client.bucket_exists(bucket).send().await?.exists {
                client.create_bucket(bucket).send().await?;
                let config = serde_json::json!({
                    "Version": "2012-10-17",
                    "Statement": [
                        {
                            "Effect": "Allow",
                            "Principal": "*",
                            "Action": [
                                "s3:GetObject"
                            ],
                            "Resource": format!("arn:aws:s3:::{}/*", bucket)
                        }
                    ]
                })
                .to_string();
                client
                    .put_bucket_policy(bucket)
                    .config(config)
                    .send()
                    .await?;
            }
        }
        client
    };
    info!("MinIO is ready");

    Ok(DatabaseConnectionMap {
        pg_conn,
        redis_conn,
        minio_conn,
    })
}
