// sea-orm entity modules follow the `<domain>/<domain>.rs` convention, which
// triggers clippy::module_inception. It is intentional and matches the Java
// package layout, so allow it crate-wide.
#![allow(clippy::module_inception)]

pub mod models;

use anyhow::{Result, anyhow};
use log::{info, warn};
use std::{sync::Arc, time::Duration};

use sea_orm::{ConnectOptions, Database, DatabaseConnection};

#[derive(Debug, Clone)]
pub struct DatabaseConnectionMap {
    pub pg_conn: DatabaseConnection,
    pub redis_conn: Option<redis::Client>,
    pub minio_conn: Option<minio::s3::MinioClient>,
}

use once_cell::sync::OnceCell;

pub static DB_CONN: OnceCell<Arc<DatabaseConnectionMap>> = OnceCell::new();

pub async fn init_db_conn() -> anyhow::Result<()> {
    let conn_map = Arc::new(build_db_map().await?);
    DB_CONN
        .set(conn_map)
        .map_err(|_| anyhow!("DB_CONN already initialized"))
}

/// Resolve the PostgreSQL schema name from the `DB_SCHEMA` env var
/// (default: `genshin_map`). The value must be a bare SQL identifier —
/// anything else falls back to the default with a warning so a hostile
/// env value can never smuggle SQL through the connection `search_path`.
pub fn default_schema() -> String {
    match std::env::var("DB_SCHEMA") {
        Ok(s) if is_valid_schema_name(&s) => s,
        Ok(s) => {
            warn!("DB_SCHEMA '{s}' is not a valid SQL identifier, falling back to 'genshin_map'");
            "genshin_map".into()
        },
        Err(_) => "genshin_map".into(),
    }
}

fn is_valid_schema_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {},
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Percent-encode a URL component (username / password / database name) so
/// credentials containing reserved characters (`@`, `:`, `/`, `#`, `?`,
/// `%`, spaces …) can't break — or smuggle extra components into — the
/// connection URL built from env vars. Both sqlx and the redis crate
/// percent-decode these components while parsing, and every unreserved
/// character encodes to itself, so already-safe values are unchanged.
pub fn encode_url_component(input: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = String::with_capacity(input.len());
    for &b in input.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(b as char)
            },
            _ => {
                out.push('%');
                out.push(HEX[(b >> 4) as usize] as char);
                out.push(HEX[(b & 0x0F) as usize] as char);
            },
        }
    }
    out
}

async fn build_db_map() -> Result<DatabaseConnectionMap> {
    // ── Postgres (required — startup fails if unreachable) ─────────────────
    let pg_conn = {
        let db_port = match std::env::var("DB_PORT") {
            Ok(v) => v
                .parse::<u16>()
                .map_err(|e| anyhow!("Invalid DB_PORT '{v}': {e}"))?,
            Err(_) => 5432,
        };
        // Credentials are percent-encoded so reserved characters in the
        // password can't break the URL; sqlx decodes them while parsing.
        let mut opt = ConnectOptions::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            encode_url_component(&std::env::var("DB_USERNAME").unwrap_or("genshin_map".into())),
            encode_url_component(&std::env::var("DB_PASSWORD").unwrap_or("".into())),
            std::env::var("DB_HOST").unwrap_or("localhost".into()),
            db_port,
            encode_url_component(&std::env::var("DB_DATABASE").unwrap_or("genshin_map".into())),
        ));
        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .acquire_timeout(Duration::from_secs(8))
            // idle/lifetime in minutes, not seconds — an 8s max_lifetime would
            // recycle every connection constantly under any load.
            .idle_timeout(Duration::from_secs(60))
            .max_lifetime(Duration::from_secs(30 * 60))
            .sqlx_logging(true)
            // Info 而非 Trace：Trace 会把绑定参数值（可能含敏感数据）打进日志。
            .sqlx_logging_level(log::LevelFilter::Info)
            // Entities carry no schema qualifier; the schema is resolved at
            // runtime through the Postgres `search_path` (DB_SCHEMA env var,
            // default `genshin_map`), applied per pooled connection.
            .set_schema_search_path(default_schema());
        Database::connect(opt).await?
    };
    info!("Postgres is ready");

    // ── Redis (optional — graceful degradation for e2e mode) ──────────────
    let redis_conn = match redis::Client::open(format!(
        "redis://{}{}@{}:{}/{}",
        encode_url_component(&std::env::var("REDIS_USERNAME").unwrap_or("".into())),
        std::env::var("REDIS_PASSWORD")
            .map(|p| format!(":{}", encode_url_component(&p)))
            .unwrap_or_default(),
        std::env::var("REDIS_HOST").unwrap_or("localhost".into()),
        std::env::var("REDIS_PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(6379),
        1,
    )) {
        Ok(client) => {
            info!("Redis is ready");
            Some(client)
        },
        Err(e) => {
            warn!("Redis connection failed, running in degraded mode: {e}");
            None
        },
    };

    // ── MinIO (optional — graceful degradation for e2e mode) ───────────────
    let minio_conn = build_minio_conn().await;

    Ok(DatabaseConnectionMap {
        pg_conn,
        redis_conn,
        minio_conn,
    })
}

/// Attempt to build a MinIO client and provision buckets. Returns `None` on
/// any failure (missing env vars, unreachable host, auth error) so the backend
/// can still start for e2e testing without a running MinIO instance.
async fn build_minio_conn() -> Option<minio::s3::MinioClient> {
    use minio::s3::types::S3Api;

    let access_key = match std::env::var("MINIO_ACCESS_KEY") {
        Ok(v) => v,
        Err(_) => {
            warn!("MINIO_ACCESS_KEY not set, skipping MinIO");
            return None;
        },
    };
    let secret_key = match std::env::var("MINIO_SECRET_KEY") {
        Ok(v) => v,
        Err(_) => {
            warn!("MINIO_SECRET_KEY not set, skipping MinIO");
            return None;
        },
    };

    let base_url = match std::env::var("MINIO_BASE_URL")
        .unwrap_or("http://localhost:9000".into())
        .parse()
    {
        Ok(url) => url,
        Err(e) => {
            warn!("MINIO_BASE_URL parse failed, skipping MinIO: {e}");
            return None;
        },
    };

    let client = match minio::s3::MinioClientBuilder::new(base_url)
        .provider(Some(minio::s3::creds::StaticProvider::new(
            &access_key,
            &secret_key,
            None,
        )))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            warn!("MinIO client build failed, skipping MinIO: {e}");
            return None;
        },
    };

    // Ensure buckets exist and set policy.
    for bucket in ["images", "bz2doc"] {
        let exists = match client.bucket_exists(bucket) {
            Ok(builder) => match builder.build().send().await {
                Ok(resp) => resp.exists(),
                Err(e) => {
                    warn!("MinIO bucket_exists failed for '{bucket}', skipping MinIO: {e}");
                    return None;
                },
            },
            Err(e) => {
                warn!("MinIO bucket_exists builder failed for '{bucket}': {e}");
                return None;
            },
        };
        if !exists {
            match client.create_bucket(bucket) {
                Ok(builder) => {
                    if let Err(e) = builder.build().send().await {
                        warn!("MinIO create_bucket '{bucket}' failed: {e}");
                        return None;
                    }
                },
                Err(e) => {
                    warn!("MinIO create_bucket builder failed for '{bucket}': {e}");
                    return None;
                },
            }
            let config = serde_json::json!({
                "Version": "2012-10-17",
                "Statement": [{
                    "Effect": "Allow",
                    "Principal": {"AWS": ["*"]},
                    "Action": ["s3:GetObject"],
                    "Resource": [format!("arn:aws:s3:::{}/*", bucket)]
                }]
            })
            .to_string();
            match client.put_bucket_policy(bucket) {
                Ok(builder) => {
                    if let Err(e) = builder.config(config).build().send().await {
                        warn!("MinIO put_bucket_policy '{bucket}' failed: {e}");
                        return None;
                    }
                },
                Err(e) => {
                    warn!("MinIO put_bucket_policy builder failed for '{bucket}': {e}");
                    return None;
                },
            }
        }
    }
    info!("MinIO is ready");
    Some(client)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_url_component_encodes_reserved_and_keeps_unreserved() {
        assert_eq!(encode_url_component("genshin_map"), "genshin_map");
        assert_eq!(
            encode_url_component("p@ss:w/rd#1?x=%"),
            "p%40ss%3Aw%2Frd%231%3Fx%3D%25"
        );
        assert_eq!(encode_url_component(""), "");
        // Unicode passwords are encoded byte-wise and decode back unchanged.
        assert_eq!(encode_url_component("密码"), "%E5%AF%86%E7%A0%81");
    }
}
