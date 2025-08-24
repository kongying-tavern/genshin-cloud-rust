mod area;
mod common_item;
mod icon_tag;
mod item;
mod marker;
mod marker_link;
mod notice;

use anyhow::Result;
use axum::{
    routing::delete,
    Router,
};

pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/icon_tag", delete(icon_tag::delete_icon_tag_cache))
        .route("/area", delete(area::delete_area_cache))
        .route("/item", delete(item::delete_item_cache))
        .route("/common_item", delete(common_item::delete_common_item_cache))
        .route("/marker", delete(marker::delete_marker_cache))
        .route("/marker_link", delete(marker_link::delete_marker_link_cache))
        .route("/notice", delete(notice::delete_notice_cache));

    Ok(ret)
}
