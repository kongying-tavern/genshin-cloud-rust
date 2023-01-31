use axum::{response::Html, routing::get, Router};

use api_base::bootstrap::{print_banner, shutdown_signal_handler};
use tracing::{Level, info};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry, fmt};

#[tokio::main]
async fn main() {
    print_banner();

    // 设置日志
    let (log_appender, _guard) =
        tracing_appender::non_blocking(tracing_appender::rolling::daily("log", "webhook-log"));
 
    Registry::default()
        .with(EnvFilter::from_default_env().add_directive(Level::DEBUG.into()))
        .with(fmt::layer().pretty())
        .with(fmt::layer().with_ansi(false).with_writer(log_appender))
        .init();

    // 路由
    let app = Router::new().route("/", get(handler));

    // 启动服务
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("服务启动，监听地址: {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal_handler())
        .await
        .unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
