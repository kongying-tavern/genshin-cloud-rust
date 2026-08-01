//! BinaryMD5 archive export for markers.
//!
//! Mirrors Java `MarkerDocController` / `MarkerDaoImpl.refreshMarkerBinaryList`.
//! Markers are grouped by `hidden_flag`; the **normal** group (flag 0) is
//! further split into pages of 3000 by `marker.id / 3000`. All other flag
//! groups are a single page (index 0).

use anyhow::{Result, anyhow};
use std::collections::BTreeMap;

use _database::{DB_CONN, models::marker::marker as marker_model};
use _utils::{db_operations::SafeEntityTrait, jwt::AuthInfo, models::wrapper::CommonResponse};

use super::binary_doc::{BinaryMd5Vo, CachedPage, get_or_compute, serialize_compress_md5};

/// Page size for the normal (flag 0) marker group.
const MARKER_PAGE_SIZE: i64 = 3000;

/// Cache key for a marker page: `marker:{flag}:{page_index}`.
fn marker_page_key(flag: i32, page_index: i64) -> String {
    format!("marker:{flag}:{page_index}")
}

/// Compute (and cache) one marker page. Returns the cached page entry.
async fn marker_page(
    flag: i32,
    page_index: i64,
    page_markers: &[&marker_model::Model],
) -> Result<CachedPage> {
    get_or_compute(marker_page_key(flag, page_index), async {
        let (compressed, md5_hex) = serialize_compress_md5(&page_markers)?;
        Ok(CachedPage {
            md5: md5_hex,
            time: chrono::Utc::now().timestamp_millis(),
            bytes: compressed,
        })
    })
    .await
}

/// `GET /marker_doc/list_page_bin_md5`
pub async fn do_list_page_bin_md5(
    _auth: AuthInfo,
    _payload: serde_json::Value,
) -> Result<CommonResponse<Vec<BinaryMd5Vo>>> {
    let db = &DB_CONN.wait().pg_conn;

    let markers = marker_model::Entity::find_safety().all(db).await?;

    // Group by hidden_flag (BTreeMap → sorted ascending)
    let mut groups: BTreeMap<i32, Vec<&marker_model::Model>> = BTreeMap::new();
    for m in &markers {
        groups.entry(m.hidden_flag as i32).or_default().push(m);
    }

    let mut result = Vec::new();
    for (flag, group_markers) in &groups {
        if *flag == HiddenFlag::Visible as i32 {
            // Normal group: split into pages of MARKER_PAGE_SIZE by id
            let mut pages: BTreeMap<i64, Vec<&marker_model::Model>> = BTreeMap::new();
            for m in group_markers {
                let page_index = m.id / MARKER_PAGE_SIZE;
                pages.entry(page_index).or_default().push(m);
            }
            for (page_index, page_markers) in &pages {
                let page = marker_page(*flag, *page_index, page_markers).await?;
                result.push(BinaryMd5Vo {
                    md5: page.md5,
                    time: page.time,
                });
            }
        } else {
            // Other flags: single page (index 0)
            let page = marker_page(*flag, 0, group_markers).await?;
            result.push(BinaryMd5Vo {
                md5: page.md5,
                time: page.time,
            });
        }
    }

    Ok(CommonResponse::new(Ok(result)))
}

/// `GET /marker_doc/list_page_bin/{md5}`
pub async fn do_list_page_bin(_auth: AuthInfo, md5: String) -> Result<Vec<u8>> {
    let db = &DB_CONN.wait().pg_conn;

    let markers = marker_model::Entity::find_safety().all(db).await?;

    let mut groups: BTreeMap<i32, Vec<&marker_model::Model>> = BTreeMap::new();
    for m in &markers {
        groups.entry(m.hidden_flag as i32).or_default().push(m);
    }

    for (flag, group_markers) in &groups {
        if *flag == HiddenFlag::Visible as i32 {
            let mut pages: BTreeMap<i64, Vec<&marker_model::Model>> = BTreeMap::new();
            for m in group_markers {
                let page_index = m.id / MARKER_PAGE_SIZE;
                pages.entry(page_index).or_default().push(m);
            }
            for (page_index, page_markers) in &pages {
                let page = marker_page(*flag, *page_index, page_markers).await?;
                if page.md5 == md5 {
                    return Ok(page.bytes);
                }
            }
        } else {
            let page = marker_page(*flag, 0, group_markers).await?;
            if page.md5 == md5 {
                return Ok(page.bytes);
            }
        }
    }

    Err(anyhow!("分页数据未生成或超出获取范围"))
}

use _utils::types::HiddenFlag;
