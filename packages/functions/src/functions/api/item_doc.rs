//! BinaryMD5 archive export for items.
//!
//! Mirrors Java `ItemDocController` / `ItemDaoImpl.refreshItemBinaryList`.
//! Items are grouped by `hidden_flag`, each group becomes one page (index 0),
//! serialized to JSON, GZIP-compressed, and keyed by the MD5 of the compressed
//! bytes.

use anyhow::{Result, anyhow};
use std::collections::BTreeMap;

use _database::{DB_CONN, models::item::item as item_model};
use _utils::{db_operations::SafeEntityTrait, jwt::AuthInfo, models::wrapper::CommonResponse};

use super::binary_doc::{BinaryMd5Vo, serialize_compress_md5};

/// `GET /item_doc/list_page_bin_md5`
///
/// Returns the MD5 list for all item pages. Each `hidden_flag` group is a
/// single page (index 0). The `time` field is the generation timestamp shared
/// by all entries.
pub async fn do_list_page_bin_md5(
    _auth: AuthInfo,
    _payload: serde_json::Value,
) -> Result<CommonResponse<Vec<BinaryMd5Vo>>> {
    let db = &DB_CONN.wait().pg_conn;
    let now = chrono::Utc::now().timestamp_millis();

    // Query all non-deleted items
    let items = item_model::Entity::find_safety().all(db).await?;

    // Group by hidden_flag (BTreeMap → sorted by flag value ascending)
    let mut groups: BTreeMap<i32, Vec<&item_model::Model>> = BTreeMap::new();
    for item in &items {
        groups
            .entry(item.hidden_flag as i32)
            .or_default()
            .push(item);
    }

    // Each group → one page → serialize + compress + MD5
    let mut result = Vec::with_capacity(groups.len());
    for group_items in groups.values() {
        let (compressed, md5_hex) = serialize_compress_md5(group_items)?;
        // NOTE: compressed bytes are discarded here — in the Java impl they are
        // cached in Caffeine keyed by md5. Without an in-process cache, the
        // list_page_bin handler regenerates on demand. A cache layer (Redis or
        // moka) should be added for production performance.
        let _ = compressed;
        result.push(BinaryMd5Vo {
            md5: md5_hex,
            time: now,
        });
    }

    Ok(CommonResponse::new(Ok(result)))
}

/// `GET /item_doc/list_page_bin/{md5}`
///
/// Returns the GZIP-compressed JSON bytes for the page whose MD5 matches.
/// Regenerates all pages and returns the matching one (no cache yet).
pub async fn do_list_page_bin(_auth: AuthInfo, md5: String) -> Result<Vec<u8>> {
    let db = &DB_CONN.wait().pg_conn;

    let items = item_model::Entity::find_safety().all(db).await?;

    // Group by hidden_flag
    let mut groups: BTreeMap<i32, Vec<&item_model::Model>> = BTreeMap::new();
    for item in &items {
        groups
            .entry(item.hidden_flag as i32)
            .or_default()
            .push(item);
    }

    // Find the page whose compressed MD5 matches
    for group_items in groups.values() {
        let (compressed, md5_hex) = serialize_compress_md5(group_items)?;
        if md5_hex == md5 {
            return Ok(compressed);
        }
    }

    Err(anyhow!("分页数据未生成或超出获取范围"))
}
