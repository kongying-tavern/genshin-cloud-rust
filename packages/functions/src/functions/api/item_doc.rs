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

use super::binary_doc::{BinaryMd5Vo, CachedPage, get_or_compute, serialize_compress_md5};

/// `GET /item_doc/list_page_bin_md5`
///
/// Returns the MD5 list for all item pages. Each `hidden_flag` group is a
/// single page (index 0). Pages are served from the in-process cache; the
/// `time` field is the page's generation timestamp (stable within the cache
/// TTL), not the request time.
pub async fn do_list_page_bin_md5(
    _auth: AuthInfo,
    _payload: serde_json::Value,
) -> Result<CommonResponse<Vec<BinaryMd5Vo>>> {
    let db = &DB_CONN.wait().pg_conn;

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

    // Each group → one page → cached (serialize + compress + MD5)
    let mut result = Vec::with_capacity(groups.len());
    for (flag, group_items) in &groups {
        let page = get_or_compute(format!("item:{flag}"), async {
            let (compressed, md5_hex) = serialize_compress_md5(group_items)?;
            Ok(CachedPage {
                md5: md5_hex,
                time: chrono::Utc::now().timestamp_millis(),
                bytes: compressed,
            })
        })
        .await?;
        result.push(BinaryMd5Vo {
            md5: page.md5,
            time: page.time,
        });
    }

    Ok(CommonResponse::new(Ok(result)))
}

/// `GET /item_doc/list_page_bin/{md5}`
///
/// Returns the GZIP-compressed JSON bytes for the page whose MD5 matches.
/// Pages are (re)generated on miss and cached, so a second fetch of the same
/// md5 hits the cache instead of re-serializing the whole dataset.
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
    for (flag, group_items) in &groups {
        let page = get_or_compute(format!("item:{flag}"), async {
            let (compressed, md5_hex) = serialize_compress_md5(group_items)?;
            Ok(CachedPage {
                md5: md5_hex,
                time: chrono::Utc::now().timestamp_millis(),
                bytes: compressed,
            })
        })
        .await?;
        if page.md5 == md5 {
            return Ok(page.bytes);
        }
    }

    Err(anyhow!("分页数据未生成或超出获取范围"))
}
