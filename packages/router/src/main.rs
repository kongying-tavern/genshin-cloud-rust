pub mod functions;
mod middlewares;
mod routes;

use anyhow::Result;
use std::io::Write;
use std::net::SocketAddr;
use std::path::Path;

use axum::serve;
use tokio::net::TcpListener;

use crate::routes::router;
use _database::init_db_conn;

/// Tee target: forwards every formatted log record to stderr (always) and to
/// a log file (only when `LOG_DIR` is set). File output is append-only, so
/// container restarts never clobber previous logs.
struct TeeTarget {
    file: Option<std::fs::File>,
}

impl Write for TeeTarget {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        std::io::stderr().write_all(buf)?;
        if let Some(f) = self.file.as_mut() {
            f.write_all(buf)?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::io::stderr().flush()?;
        if let Some(f) = self.file.as_mut() {
            f.flush()?;
        }
        Ok(())
    }
}

/// Open `<LOG_DIR>/genshin-cloud.log` in append mode. Returns `None` (with a
/// warning) when `LOG_DIR` is unset or the file cannot be opened — logging
/// then stays on stderr only, and startup never fails because of logs.
fn open_log_file() -> Option<std::fs::File> {
    let dir = std::env::var("LOG_DIR").ok()?;
    // Create the directory when it doesn't exist yet (containers often mount
    // an empty volume; a missing dir should not silently disable file logs).
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("LOG_DIR set but {dir} cannot be created, logging to stderr only: {e}");
        return None;
    }
    let path = Path::new(&dir).join("genshin-cloud.log");
    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        Ok(f) => {
            eprintln!("log file output enabled: {}", path.display());
            Some(f)
        },
        Err(e) => {
            eprintln!(
                "LOG_DIR set but {} open failed, logging to stderr only: {e}",
                path.display()
            );
            None
        },
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    // Install the ring crypto provider for jsonwebtoken (v10 requires an
    // explicit process-level CryptoProvider). This must happen before any
    // JWT encode/decode call. We use ring (not aws-lc-rs) to stay consistent
    // with the workspace's aws-lc-free rustls+ring policy.
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let mut builder = env_logger::Builder::new();
    builder.filter(None, log::LevelFilter::Info);
    if let Some(file) = open_log_file() {
        builder.target(env_logger::Target::Pipe(Box::new(TeeTarget {
            file: Some(file),
        })));
    }
    builder.init();

    // Fail fast on a missing JWT secret. The lazy JWT key statics would
    // otherwise panic on the first authenticated request — and with
    // `panic = "abort"` in the release profile that kills the whole process
    // mid-traffic instead of refusing to start with a clear message.
    if std::env::var("JWT_SECRET").is_err() {
        anyhow::bail!(
            "JWT_SECRET must be set (see .env.example); generate one with: openssl rand -base64 48"
        );
    }

    let port = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80);

    log::info!("Site will run on port {}", port);
    init_db_conn().await?;

    let router = router()
        .await?
        .into_make_service_with_connect_info::<SocketAddr>();

    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("Failed to bind");
    serve(listener, router).await?;

    Ok(())
}
