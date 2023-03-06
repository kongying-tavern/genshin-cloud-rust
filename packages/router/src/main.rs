use axum::{routing::get, Router};
use hyper::server::Server;
use log::info;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use _functions::query_all_positions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .init();

    let port = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80);

    let db_conn = _database::init(_database::DatabaseNetworkConfig {
        host: "localhost".into(),
        port: 5432,
        username: std::env::var("POSTGRES_USER").unwrap_or("genshin_map".into()),
        password: std::env::var("POSTGRES_PASSWORD").unwrap_or("root".into()),
    })
    .await?;

    let middleware_stack = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .into_inner();

    let app = Router::new()
        .route(
            "/test",
            get(|| async {
                match query_all_positions(db_conn).await {
                    Ok(res) => res,
                    Err(e) => format!("{:?}", e),
                }
            }),
        )
        .layer(middleware_stack);

    info!("Site will run on port {}", port);

    Server::bind(&format!("0.0.0.0:{}", port).parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
