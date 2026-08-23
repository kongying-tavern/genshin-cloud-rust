//! BinaryMD5 archive export for markers.
//!
//! Mirrors Java `MarkerDocController` / `MarkerDaoImpl.refreshMarkerBinaryList`.
//! Markers are grouped by `hidden_flag`; the **normal** group (flag 0) is
//! further split into pages of 3000 by `marker.id / 3000`. All other flag
//! groups are a single page (index 0).
//!
//! A result-level cache avoids re-scanning the whole table on every request.

use anyhow::{Result, anyhow};
use std::collections::BTreeMap;

use _database::{DB_CONN, models::marker::marker as marker_model};
use _utils::{jwt::AuthInfo, models::wrapper::CommonResponse};
use sea_orm::{ColumnTrait, QueryFilter, QuerySelect, prelude::*};

use super::binary_doc::{
    BinaryMd5Vo, CachedPage, ResultEntry, get_or_compute, get_result_cached, serialize_compress_md5,
};

use _utils::types::HiddenFlag;

/// Page size for the normal (flag 0) marker group.
const MARKER_PAGE_SIZE: i64 = 3000;

/// Cache key for a marker page: `marker:{flag}:{page_index}`.
fn marker_page_key(flag: i32, page_index: i64) -> String {
    format!("marker:{flag}:{page_index}")
}

/// `GET /marker_doc/list_page_bin_md5`
pub async fn do_list_page_bin_md5(
    auth: AuthInfo,
    _payload: serde_json::Value,
) -> Result<CommonResponse<Vec<BinaryMd5Vo>>> {
    // 可见性（Java listMarkerBinaryMD5）：低等级用户拿不到高等级分组的
    // md5，隐藏/测试服页对普通用户如同不存在。
    let allowed = _utils::types::allowed_hidden_flags(auth.info.role_id);
    let entries = marker_result().await?;
    Ok(CommonResponse::new(Ok(entries
        .iter()
        .filter(|e| allowed.contains(&entry_flag(&e.key)))
        .map(|e| e.vo.clone())
        .collect())))
}

/// `GET /marker_doc/list_page_bin/{md5}`
pub async fn do_list_page_bin(auth: AuthInfo, md5: String) -> Result<Vec<u8>> {
    // 与 md5 清单同口径的角色过滤：知道 md5 也不能取到越权分组。
    let allowed = _utils::types::allowed_hidden_flags(auth.info.role_id);
    let entries = marker_result().await?;
    entries
        .iter()
        .find(|e| e.vo.md5 == md5 && allowed.contains(&entry_flag(&e.key)))
        .map(|e| e.bytes.clone())
        .ok_or_else(|| anyhow!("分页数据未生成或超出获取范围"))
}

/// Cache key 形如 `marker:{flag}:{page_index}` —— 解析其中的 flag。
fn entry_flag(key: &str) -> i32 {
    key.split(':')
        .nth(1)
        .and_then(|f| f.parse().ok())
        .unwrap_or(-1)
}

/// Compute (and cache) the full marker page set.
async fn marker_result() -> Result<Vec<ResultEntry>> {
    let db = &DB_CONN.wait().pg_conn;

    get_result_cached("marker:result".into(), async {
        let markers = marker_model::Entity::find()
            .select_only()
            .column(marker_model::Column::Version)
            .column(marker_model::Column::Id)
            .column(marker_model::Column::CreateTime)
            .column(marker_model::Column::UpdateTime)
            .column(marker_model::Column::CreatorId)
            .column(marker_model::Column::UpdaterId)
            .column(marker_model::Column::DelFlag)
            .column(marker_model::Column::MarkerStamp)
            .column(marker_model::Column::MarkerTitle)
            .column(marker_model::Column::Position)
            .column(marker_model::Column::Content)
            .column(marker_model::Column::Picture)
            .column(marker_model::Column::MarkerCreatorId)
            .column(marker_model::Column::PictureCreatorId)
            .column(marker_model::Column::VideoPath)
            .column(marker_model::Column::RefreshTime)
            .column(marker_model::Column::HiddenFlag)
            .column(marker_model::Column::Extra)
            .filter(marker_model::Column::DelFlag.eq(false))
            .all(db)
            .await?;
        let ids: Vec<i64> = markers.iter().map(|m| m.id).collect();
        let item_map = super::marker::marker_item_map(db, &ids).await?;
        let linkage_map = super::marker::marker_linkage_map(db, &ids).await?;

        // Group by hidden_flag (BTreeMap → sorted ascending)
        let mut groups: BTreeMap<i32, Vec<&marker_model::Model>> = BTreeMap::new();
        for m in &markers {
            groups.entry(m.hidden_flag as i32).or_default().push(m);
        }

        let mut entries = Vec::new();
        for (flag, group_markers) in &groups {
            if *flag == HiddenFlag::Visible as i32 {
                // Normal group: split into pages of MARKER_PAGE_SIZE by id
                let mut pages: BTreeMap<i64, Vec<&marker_model::Model>> = BTreeMap::new();
                for m in group_markers {
                    let page_index = m.id / MARKER_PAGE_SIZE;
                    pages.entry(page_index).or_default().push(m);
                }
                for (page_index, page_markers) in &pages {
                    let key = marker_page_key(*flag, *page_index);
                    let page = get_or_compute(key.clone(), async {
                        // camelCase `MarkerVO` naming (Java `MarkerVo` wire
                        // contract) — snake_case models would break the
                        // frontend parser.
                        let vos: Vec<_> = page_markers
                            .iter()
                            .map(|m| {
                                super::marker::model_to_vo_doc(m, &item_map, Some(&linkage_map))
                            })
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
            } else {
                // Other flags: single page (index 0)
                let key = marker_page_key(*flag, 0);
                let page = get_or_compute(key.clone(), async {
                    let vos: Vec<_> = group_markers
                        .iter()
                        .map(|m| super::marker::model_to_vo_doc(m, &item_map, Some(&linkage_map)))
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
        }
        Ok(entries)
    })
    .await
}
