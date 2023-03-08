use anyhow::Result;
use axum::Router;
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

mod area;
mod history;
mod icon;
mod icon_type;
mod item;
mod item_common;
mod item_doc;
mod item_type;
mod marker;
mod marker_doc;
mod punctuate;
mod punctuate_audit;
mod route;
mod score;
mod tag;
mod tag_doc;
mod tag_type;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct PageSearchParams {
    pub current: Option<i64>,
    pub size: Option<i64>,
}

pub async fn register() -> Result<Router> {
    let middleware_stack = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .into_inner();

    let router = Router::new()
        .nest("/area", area::register().await?)
        .nest("/history", history::register().await?)
        .nest("/icon", icon::register().await?)
        .nest("/icon_type", icon_type::register().await?)
        .nest("/item_common", item_common::register().await?)
        .nest("/item_doc", item_doc::register().await?)
        .nest("/item_type", item_type::register().await?)
        .nest("/item", item::register().await?)
        .nest("/marker_doc", marker_doc::register().await?)
        .nest("/marker", marker::register().await?)
        .nest("/punctuate_audit", punctuate_audit::register().await?)
        .nest("/punctuate", punctuate::register().await?)
        .nest("/route", route::register().await?)
        .nest("/score", score::register().await?)
        .nest("/tag_doc", tag_doc::register().await?)
        .nest("/tag_type", tag_type::register().await?)
        .nest("/tag", tag::register().await?)
        .layer(middleware_stack);
    Ok(router)
}
