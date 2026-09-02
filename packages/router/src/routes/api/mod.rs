pub mod app;
pub mod area;
pub mod cache;
pub mod history;
pub mod icon;
pub mod icon_doc;
pub mod icon_type;
pub mod item;
pub mod item_common;
pub mod item_doc;
pub mod item_type;
pub mod marker;
pub mod marker_doc;
pub mod marker_link;
pub mod marker_link_doc;
pub mod notice;
pub mod res;
// 已弃用（对齐决策，见 docs/zh-chs/guides/sync-with-java-roadmap.md）：
// Java 的 /api/route（RouteController）与 /api/punctuate、/api/punctuate_audit
// 三域不在本后端提供；route 模块与路由挂载因此保持注释状态。
// pub mod route;
pub mod score;
pub mod tag;
pub mod tag_doc;
pub mod tag_type;

use anyhow::Result;

use axum::{Router, middleware::from_extractor};

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .nest("/app", app::router().await?)
        .nest("/area", area::router().await?)
        .nest("/cache", cache::router().await?)
        .nest("/history", history::router().await?)
        .nest("/icon", icon::router().await?)
        .nest("/icon_doc", icon_doc::router().await?)
        .nest("/icon_type", icon_type::router().await?)
        .nest("/item", item::router().await?)
        .nest("/item_common", item_common::router().await?)
        .nest("/item_doc", item_doc::router().await?)
        .nest("/item_type", item_type::router().await?)
        .nest("/marker", marker::router().await?)
        .nest("/marker_doc", marker_doc::router().await?)
        .nest("/marker_link", marker_link::router().await?)
        .nest("/marker_link_doc", marker_link_doc::router().await?)
        .nest("/notice", notice::router().await?)
        .nest("/res", res::router().await?)
        // .nest("/route", route::router().await?)
        .nest("/score", score::router().await?)
        .nest("/tag", tag::router().await?)
        .nest("/tag_doc", tag_doc::router().await?)
        .nest("/tag_type", tag_type::router().await?)
        .layer(from_extractor::<crate::middlewares::ExtractAuthInfo>());

    Ok(ret)
}
