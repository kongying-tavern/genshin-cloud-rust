//! Cache invalidation endpoints.
//!
//! In the Java backend, these wire to `CacheService.clean*` which evicts
//! Caffeine in-process cache entries. The Rust backend uses Redis for caching
//! (not in-process Caffeine), but the Redis read-through cache layer and the
//! BinaryMD5 `*_doc` archive pipeline are not yet implemented — so there is
//! currently no cache to invalidate.
//!
//! These handlers are honest no-ops: they return success without claiming to
//! have cleared anything. Once the Redis cache layer is wired (keyed by
//! `cache:{domain}` patterns), each handler should issue a `DEL cache:{domain}`
//! against `DB_CONN.redis_conn`.

use anyhow::Result;

use _utils::{
    jwt::AuthInfo,
    models::{common::EmptyResponse, wrapper::CommonResponse},
};

/// 清除地区缓存。当前无缓存层，为 no-op。
pub async fn do_delete_area_cache(_auth: AuthInfo) -> Result<CommonResponse<EmptyResponse>> {
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

/// 清除物品缓存。当前无缓存层，为 no-op。
pub async fn do_delete_item_cache(_auth: AuthInfo) -> Result<CommonResponse<EmptyResponse>> {
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

/// 清除公共物品缓存。当前无缓存层，为 no-op。
pub async fn do_delete_common_item_cache(_auth: AuthInfo) -> Result<CommonResponse<EmptyResponse>> {
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

/// 清除图标标签缓存。当前无缓存层，为 no-op。
pub async fn do_delete_icon_tag_cache(_auth: AuthInfo) -> Result<CommonResponse<EmptyResponse>> {
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

/// 清除点位缓存。当前无缓存层，为 no-op。
pub async fn do_delete_marker_cache(_auth: AuthInfo) -> Result<CommonResponse<EmptyResponse>> {
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

/// 清除点位连线缓存。当前无缓存层，为 no-op。
pub async fn do_delete_marker_link_cache(_auth: AuthInfo) -> Result<CommonResponse<EmptyResponse>> {
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

/// 清除公告缓存。当前无缓存层，为 no-op。
pub async fn do_delete_notice_cache(_auth: AuthInfo) -> Result<CommonResponse<EmptyResponse>> {
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}
