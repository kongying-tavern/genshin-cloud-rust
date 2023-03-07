use axum::Router;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

pub mod area;

pub async fn register() -> Result<Router, Box<dyn std::error::Error>> {
    let middleware_stack = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .into_inner();

    let router = Router::new()
        .nest("/area", area::register().await?)
        .layer(middleware_stack);
    Ok(router)
}
