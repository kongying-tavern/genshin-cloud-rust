use anyhow::Result;
use std::io::Write;
use std::net::SocketAddr;
use std::path::Path;

use axum::serve;
use tokio::net::TcpListener;

// The route table and middlewares live in the crate's lib target so the
// integration tests can drive the assembled Router directly (oneshot); this
// binary only does process-level wiring around it.
use _database::init_db_conn;
use _router::routes::router;

/// 已知占位符 JWT_SECRET（比较时去空白、转小写）：示例/文档值被原样复制
/// 进生产配置是最常见的密钥泄漏来源（看过文档的人都能伪造 token），
/// 启动期一律拒绝。只影响启动检查，不改运行时 getter（集成测试在进程内
/// 设了短 secret）。
const JWT_SECRET_PLACEHOLDERS: [&str; 10] = [
    "change-me-to-a-strong-random-secret",
    "change_me_to_a_long_random_string",
    "changeme",
    "change-me",
    "secret",
    "dev-secret",
    "test-secret",
    "123456",
    "your-secret-key",
    "your_jwt_secret",
];

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

/// 优雅停机信号源：Ctrl+C 或（Unix 下）SIGTERM 二者先到者。
///
/// 生产容器里 tini 把 `docker stop` 的 SIGTERM 转发给本进程（见 Dockerfile
/// 的 tini 注释），本函数兑现其宣称的优雅停机：收到信号后**先**广播关闭
/// 全部 WebSocket 会话（`begin_shutdown`），再返回交给 axum 的 graceful
/// shutdown 等待 HTTP 在途请求排空。axum 会等所有连接关闭才退出，长连
/// WS 若不主动断开会把进程在停机窗口内一直挂住——所以 WS 侧必须配合
/// 先关（停机广播的实现见 `_functions::functions::ws`）。
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            },
            // 信号处理注册失败（极端环境）：该分支永不成真，仅剩 Ctrl+C 可触发。
            Err(_) => std::future::pending::<()>().await,
        }
    };

    // Windows 无 SIGTERM：用永不成真的 future 占位，保持 select 形状一致，
    // 同一份代码在两个平台都能编译（CI 含 windows-latest 测试 job）。
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    _functions::functions::ws::begin_shutdown().await;
    log::info!("shutdown signal received, WS sessions closed, draining in-flight requests");
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

    // 强度校验：只查存在不查强度等于放行弱密钥——短密钥可被离线暴力破解，
    // 占位符值则任何看过文档的人都能伪造 token。阈值 32 字符与占位符集合
    // 均为启动期硬拒绝（对齐上方缺失即 bail 的 fail-fast 风格）。
    if let Ok(secret) = std::env::var("JWT_SECRET") {
        let trimmed = secret.trim();
        let normalized = trimmed.to_ascii_lowercase();
        if trimmed.len() < 32 || JWT_SECRET_PLACEHOLDERS.contains(&normalized.as_str()) {
            anyhow::bail!(
                "JWT_SECRET is too weak: use at least 32 random characters and never a placeholder value (see .env.example); generate one with: openssl rand -base64 48"
            );
        }
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
    // 优雅停机：tini 转发的 SIGTERM / Ctrl+C 触发 shutdown_signal——先广播
    // 关闭 WS 会话，再等 HTTP 在途请求排空后退出（细节见 shutdown_signal）。
    serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
