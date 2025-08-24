mod api;
mod system;

use anyhow::Result;

use axum::{
    extract::DefaultBodyLimit, http::StatusCode, middleware::from_extractor,
    response::IntoResponse, routing::post, Router,
};

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/oauth/token", post(system::oauth::oauth))
        .merge(system::router().await?)
        .merge(api::router().await?)
        .fallback(|| async { (StatusCode::NOT_IMPLEMENTED, "Not Implemented").into_response() })
        .layer(from_extractor::<crate::middlewares::ExtractUserAgent>())
        .layer(from_extractor::<crate::middlewares::ExtractIP>())
        .layer(DefaultBodyLimit::max(1024 * 1024 * 16)); // 16 MiB

    Ok(ret)
}
