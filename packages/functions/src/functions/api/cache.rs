//! Cache invalidation endpoints.
//!
//! In the Java backend, these wire to `CacheService.clean*` which evicts
//! Caffeine in-process cache entries. The Rust backend keeps the BinaryMD5
//! `*_doc` pages in an in-process moka cache (see `binary_doc`), so the item /
//! marker / marker-link refresh handlers flush it. The other domains
//! (area / common_item / icon_tag / notice) have no in-process cache yet, so
//! their handlers remain honest no-ops.

use anyhow::Result;

use _database::DatabaseConnectionMap;
use _utils::{jwt::AuthInfo, models::wrapper::CommonResponse};

use super::binary_doc;

/// 清除地区缓存。当前无对应缓存层，为 no-op。
pub async fn do_delete_area_cache(auth: AuthInfo) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    Ok(CommonResponse::new(Ok(true)))
}

/// 清除物品缓存（BinaryMD5 item 页）。
pub async fn do_delete_item_cache(
    db: &DatabaseConnectionMap,
    auth: AuthInfo,
) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    binary_doc::invalidate_all(db).await;
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

/// 清除图标缓存（Java `CacheController` `DELETE /api/cache/icon`，
/// body `List<Long>` 图标 ID 列表，空列表 = 全量清除）。
///
/// Java 侧按 ID 精确驱逐 Caffeine 图标缓存；Rust 的 icon 缓存是整块
/// blob（icon_doc / 二进制导出），无法按单 ID 部分失效——空列表与非空
/// 列表都整体失效（blob 中含所请求的 ID，粗粒度失效保持一致性）。
/// 依赖方拉取新 blob 后自然拿到最新图标集。
pub async fn do_delete_icon_cache(
    db: &DatabaseConnectionMap,
    auth: AuthInfo,
    _icon_ids: Vec<i64>,
) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    binary_doc::invalidate_all(db).await;
    super::super::ws::ws_broadcast("IconBinaryPurged", serde_json::Value::Null);
    Ok(CommonResponse::new(Ok(true)))
}

/// 清除点位缓存（BinaryMD5 marker 页）。
pub async fn do_delete_marker_cache(
    db: &DatabaseConnectionMap,
    auth: AuthInfo,
) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    binary_doc::invalidate_all(db).await;
    super::super::ws::ws_broadcast("MarkerBinaryPurged", serde_json::Value::Null);
    Ok(CommonResponse::new(Ok(true)))
}

/// 清除点位连线缓存（BinaryMD5 link list/graph 页）。
pub async fn do_delete_marker_link_cache(
    db: &DatabaseConnectionMap,
    auth: AuthInfo,
) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    binary_doc::invalidate_all(db).await;
    super::super::ws::ws_broadcast("MarkerLinkageBinaryPurged", serde_json::Value::Null);
    Ok(CommonResponse::new(Ok(true)))
}

/// 清除公告缓存。当前无对应缓存层，为 no-op。
pub async fn do_delete_notice_cache(auth: AuthInfo) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    Ok(CommonResponse::new(Ok(true)))
}
