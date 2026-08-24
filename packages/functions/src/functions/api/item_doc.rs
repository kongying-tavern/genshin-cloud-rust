//! BinaryMD5 archive export for items.
//!
//! Mirrors Java `ItemDocController` / `ItemDaoImpl.refreshItemBinaryList`.
//! Items are grouped by `hidden_flag`, each group becomes one page (index 0),
//! serialized to JSON, GZIP-compressed, and keyed by the MD5 of the compressed
//! bytes.
//!
//! A result-level cache avoids re-scanning the whole table on every request:
//! a warm `get_result_cached("item:result", ...)` performs zero DB queries.

use anyhow::{Result, anyhow};
use std::collections::BTreeMap;

use _database::{DB_CONN, models::item::item as item_model};
use _utils::{db_operations::SafeEntityTrait, jwt::AuthInfo, models::wrapper::CommonResponse};

use super::binary_doc::{
    BinaryMd5Vo, CachedPage, ResultEntry, get_or_compute, get_result_cached, serialize_compress_md5,
};

use super::item::{item_to_vo, marker_count_map, type_id_map};

/// `GET /item_doc/list_page_bin_md5`
///
/// Returns the MD5 list for all item pages. Each `hidden_flag` group is a
/// single page (index 0). Served from the result cache — no DB scan on a
/// warm hit.
pub async fn do_list_page_bin_md5(
    auth: AuthInfo,
    _payload: serde_json::Value,
) -> Result<CommonResponse<Vec<BinaryMd5Vo>>> {
    // 可见性（Java ItemDoc）：低等级用户拿不到高等级分组的 md5。
    let allowed = _utils::types::allowed_hidden_flags(auth.info.role_id);
    let entries = item_result().await?;
    Ok(CommonResponse::new(Ok(entries
        .iter()
        .filter(|e| allowed.contains(&entry_flag(&e.key)))
        .map(|e| e.vo.clone())
        .collect())))
}

/// `GET /item_doc/list_page_bin/{md5}`
///
/// Returns the GZIP-compressed JSON bytes for the page whose MD5 matches.
/// Served from the result cache — no DB scan on a warm hit.
pub async fn do_list_page_bin(auth: AuthInfo, md5: String) -> Result<Vec<u8>> {
    // 与 md5 清单同口径的角色过滤：知道 md5 也不能取到越权分组。
    let allowed = _utils::types::allowed_hidden_flags(auth.info.role_id);
    let entries = item_result().await?;
    entries
        .iter()
        .find(|e| e.vo.md5 == md5 && allowed.contains(&entry_flag(&e.key)))
        .map(|e| e.bytes.clone())
        .ok_or_else(|| anyhow!("分页数据未生成或超出获取范围"))
}

/// Cache key 形如 `item:{flag}` —— 解析其中的 flag。
fn entry_flag(key: &str) -> i32 {
    key.split(':')
        .nth(1)
        .and_then(|f| f.parse().ok())
        .unwrap_or(-1)
}

/// Compute (and cache) the full item page set.
async fn item_result() -> Result<Vec<ResultEntry>> {
    let db = &DB_CONN.wait().pg_conn;

    get_result_cached("item:result".into(), async {
        // Query all non-deleted items (once per TTL window)
        let items = item_model::Entity::find_safety().all(db).await?;
        let type_map = type_id_map(db).await?;
        let icon_map = super::icon::icon_tag_map(db).await?;
        let count_map = marker_count_map(db).await?;

        // Group by hidden_flag (BTreeMap → sorted by flag value ascending)
        let mut groups: BTreeMap<i32, Vec<&item_model::Model>> = BTreeMap::new();
        for item in &items {
            groups
                .entry(item.hidden_flag as i32)
                .or_default()
                .push(item);
        }

        // Each group → one page → cached (serialize + compress + MD5).
        // The blob is serialized through the camelCase `ItemVO` (the wire
        // contract of the item_doc pages is the Java `ItemVo` naming —
        // snake_case models would break the frontend parser).
        let mut entries = Vec::with_capacity(groups.len());
        for (flag, group_items) in &groups {
            let key = format!("item:{flag}");
            let page = get_or_compute(key.clone(), async {
                let vos: Vec<_> = group_items
                    .iter()
                    .map(|m| item_to_vo(m, &type_map, &icon_map, &count_map))
                    .collect();
                let (compressed, md5_hex) = serialize_compress_md5(&vos)?;
                Ok(CachedPage {
                    md5: md5_hex,
                    time: chrono::Utc::now().timestamp_millis(),
                    bytes: compressed,
                })
            })
            .await?;
            entries.push(ResultEntry {
                key,
                vo: BinaryMd5Vo {
                    md5: page.md5,
                    time: page.time,
                },
                bytes: page.bytes,
            });
        }
        Ok(entries)
    })
    .await
}
