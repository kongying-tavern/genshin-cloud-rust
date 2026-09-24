//! Cache invalidation endpoints.
//!
//! In the Java backend, these wire to `CacheService.clean*` which evicts
//! Caffeine in-process cache entries. The Rust backend keeps the BinaryMD5
//! `*_doc` pages in an in-process moka cache (see `binary_doc`), so the item /
//! marker / marker-link refresh handlers flush it. The other domains
//! (area / common_item / icon_tag / notice) have no in-process cache yet, so
//! their handlers remain honest no-ops.

use anyhow::Result;

use _utils::{jwt::AuthInfo, models::wrapper::CommonResponse};

use super::binary_doc;

/// 清除地区缓存。当前无对应缓存层，为 no-op。
pub async fn do_delete_area_cache(auth: AuthInfo) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    Ok(CommonResponse::new(Ok(true)))
}

/// 清除物品缓存（BinaryMD5 item 页）。
pub async fn do_delete_item_cache(auth: AuthInfo) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    binary_doc::invalidate_all().await;
    super::super::ws::ws_broadcast("ItemBinaryPurged", serde_json::Value::Null);
    Ok(CommonResponse::new(Ok(true)))
}

/// 清除公共物品缓存。当前无对应缓存层，为 no-op。
pub async fn do_delete_common_item_cache(auth: AuthInfo) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    Ok(CommonResponse::new(Ok(true)))
}

/// 清除图标标签缓存。当前无对应缓存层，为 no-op。
/// 清除标签缓存。当前无对应缓存层，为 no-op（已读取请求体 tag 列表）。
pub async fn do_delete_icon_tag_cache(
    auth: AuthInfo,
    _tags: Vec<String>,
) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    super::super::ws::ws_broadcast("IconTagBinaryPurged", serde_json::Value::Null);
    Ok(CommonResponse::new(Ok(true)))
}

/// 清除点位缓存（BinaryMD5 marker 页）。
pub async fn do_delete_marker_cache(auth: AuthInfo) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    binary_doc::invalidate_all().await;
    super::super::ws::ws_broadcast("MarkerBinaryPurged", serde_json::Value::Null);
    Ok(CommonResponse::new(Ok(true)))
}

/// 清除点位连线缓存（BinaryMD5 link list/graph 页）。
pub async fn do_delete_marker_link_cache(auth: AuthInfo) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    binary_doc::invalidate_all().await;
    super::super::ws::ws_broadcast("MarkerLinkageBinaryPurged", serde_json::Value::Null);
    Ok(CommonResponse::new(Ok(true)))
}

/// 清除公告缓存。当前无对应缓存层，为 no-op。
pub async fn do_delete_notice_cache(auth: AuthInfo) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    Ok(CommonResponse::new(Ok(true)))
}
