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

use _database::{
    DB_CONN,
    models::{
        item::item as item_model,
        marker::{marker as marker_model, marker_item_link as mil_model},
    },
};
use _utils::{db_operations::SafeEntityTrait, jwt::AuthInfo, models::wrapper::CommonResponse};
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

// ---- 差异比对快照（Java `MarkerDocService.getMarkerDiffSnapshot`） ----

/// `MarkerDiffSnapshotVo { uint64 version = 1; uint64 id = 2;
/// optional string linkage_id = 15; }` 的一行快照。
pub(crate) struct DiffSnapshot {
    pub version: u64,
    pub id: u64,
    /// Java 侧 `isNotBlank` 守卫：空串视为未设置。
    pub linkage_id: Option<String>,
}

/// proto3 varint（LEB128）编码。
fn put_varint(buf: &mut Vec<u8>, mut v: u64) {
    loop {
        let byte = (v & 0x7f) as u8;
        v >>= 7;
        if v == 0 {
            buf.push(byte);
            break;
        }
        buf.push(byte | 0x80);
    }
}

/// protobuf tag = field_number << 3 | wire_type（0 = varint，2 = LEN）。
fn tag(field: u64, wire: u64) -> u64 {
    (field << 3) | wire
}

/// 单个快照消息的 wire 字节。proto3 隐式存在语义：等于默认值（0）的
/// 标量字段不落 wire，与 Java protobuf 的序列化字节保持一致。
fn encode_snapshot(s: &DiffSnapshot) -> Vec<u8> {
    let mut msg = Vec::new();
    if s.version != 0 {
        put_varint(&mut msg, tag(1, 0));
        put_varint(&mut msg, s.version);
    }
    if s.id != 0 {
        put_varint(&mut msg, tag(2, 0));
        put_varint(&mut msg, s.id);
    }
    if let Some(lid) = s.linkage_id.as_deref().filter(|l| !l.is_empty()) {
        put_varint(&mut msg, tag(15, 2));
        put_varint(&mut msg, lid.len() as u64);
        msg.extend_from_slice(lid.as_bytes());
    }
    msg
}

/// `MarkerDiffSnapshotVoList { repeated MarkerDiffSnapshotVo snapshots = 1; }`
pub(crate) fn encode_diff_snapshot_list(snapshots: &[DiffSnapshot]) -> Vec<u8> {
    let mut out = Vec::new();
    for s in snapshots {
        let msg = encode_snapshot(s);
        put_varint(&mut out, tag(1, 2));
        put_varint(&mut out, msg.len() as u64);
        out.extend_from_slice(&msg);
    }
    out
}

/// `GET /marker_doc/list_diff_snapshot`
///
/// Java `MarkerDocService.getMarkerDiffSnapshot`：走 `searchMarker`
/// （areaIdList = 全部地区 → 物品按可见 flag 过滤 → marker_item_link）
/// 取点位集合，输出**未压缩**的 `MarkerDiffSnapshotVoList` protobuf 字节
/// （每点位 {version, id, linkageId}），供前端与本地缓存做增量比对。
/// 与 bin 页不同，此处不 GZIP —— 前端直接 `arrayBuffer` 后交给
/// protobuf 解码器。
pub async fn do_list_diff_snapshot(auth: AuthInfo) -> Result<Vec<u8>> {
    // 可见性（Java HiddenFlagEnum.getFlagListByMask(userDataLevel)）
    let allowed = _utils::types::allowed_hidden_flags(auth.info.role_id);
    let db = &DB_CONN.wait().pg_conn;

    // 结果按可见 flag 集缓存（moka，TTL 3600s）；marker / linkage 写路径
    // 的 invalidate_doc_cache() 会整体失效本键。
    let mut flags = allowed.clone();
    flags.sort_unstable();
    let key = format!(
        "marker:diff_snapshot:{}",
        flags
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    let page = get_or_compute(key, async {
        let bytes = diff_snapshot_bytes(db, &allowed).await?;
        let digest = md5::compute(&bytes);
        Ok(CachedPage {
            md5: format!("{:x}", digest),
            time: chrono::Utc::now().timestamp_millis(),
            bytes,
        })
    })
    .await?;
    Ok(page.bytes)
}

/// Java `searchMarkerId`（areaIdList = 全部地区）+ `listMarkerById`：
/// 物品按可见 flag 过滤 → marker_item_link → 点位（del_flag=false 且
/// 可见 flag）→ (version, id) 集合，再附上每点位首个关联组 ID。
async fn diff_snapshot_bytes(db: &sea_orm::DatabaseConnection, allowed: &[i32]) -> Result<Vec<u8>> {
    let item_ids: Vec<i64> = item_model::Entity::find_safety()
        .filter(item_model::Column::HiddenFlag.is_in(allowed.to_vec()))
        .select_only()
        .column(item_model::Column::Id)
        .into_tuple::<i64>()
        .all(db)
        .await?;
    let mut marker_ids: Vec<i64> = Vec::new();
    for chunk in item_ids.chunks(1000) {
        marker_ids.extend(
            mil_model::Entity::find_safety()
                .filter(mil_model::Column::ItemId.is_in(chunk))
                .select_only()
                .column(mil_model::Column::MarkerId)
                .into_tuple::<i64>()
                .all(db)
                .await?,
        );
    }
    marker_ids.sort_unstable();
    marker_ids.dedup();

    let mut markers: Vec<(i64, i64)> = Vec::new();
    for chunk in marker_ids.chunks(1000) {
        markers.extend(
            marker_model::Entity::find_safety()
                .filter(marker_model::Column::Id.is_in(chunk))
                .filter(marker_model::Column::HiddenFlag.is_in(allowed.to_vec()))
                .select_only()
                .column(marker_model::Column::Version)
                .column(marker_model::Column::Id)
                .into_tuple::<(i64, i64)>()
                .all(db)
                .await?,
        );
    }
    let ids: Vec<i64> = markers.iter().map(|(_, id)| *id).collect();
    let linkage_map = super::marker::marker_linkage_map(db, &ids).await?;

    let snapshots: Vec<DiffSnapshot> = markers
        .into_iter()
        .map(|(version, id)| DiffSnapshot {
            version: version as u64,
            id: id as u64,
            linkage_id: linkage_map.get(&id).cloned(),
        })
        .collect();
    Ok(encode_diff_snapshot_list(&snapshots))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(version: u64, id: u64, linkage: Option<&str>) -> DiffSnapshot {
        DiffSnapshot {
            version,
            id,
            linkage_id: linkage.map(str::to_string),
        }
    }

    #[test]
    fn snapshot_wire_matches_hand_computed_bytes() {
        // {version:5, id:123, linkage_id:"abc"}：
        // 08 05 (field1 varint) | 10 7b (field2 varint) |
        // 7a 03 61 62 63 (field15 LEN) —— 9 字节消息体。
        assert_eq!(
            encode_snapshot(&snap(5, 123, Some("abc"))),
            vec![0x08, 0x05, 0x10, 0x7b, 0x7a, 0x03, b'a', b'b', b'c']
        );
    }

    #[test]
    fn snapshot_omits_blank_linkage_id() {
        // Java isNotBlank 守卫：空串与缺失都不写 field 15。
        assert_eq!(
            encode_snapshot(&snap(5, 123, None)),
            vec![0x08, 0x05, 0x10, 0x7b]
        );
        assert_eq!(
            encode_snapshot(&snap(5, 123, Some(""))),
            vec![0x08, 0x05, 0x10, 0x7b]
        );
    }

    #[test]
    fn snapshot_skips_default_scalars_and_encodes_multibyte_varints() {
        // proto3 隐式存在：version=0 不落 wire；id=300 → varint ac 02。
        assert_eq!(encode_snapshot(&snap(0, 300, None)), vec![0x10, 0xac, 0x02]);
    }

    #[test]
    fn list_wraps_each_snapshot_in_field1() {
        assert_eq!(
            encode_diff_snapshot_list(&[snap(5, 123, Some("abc"))]),
            vec![
                0x0a, 0x09, 0x08, 0x05, 0x10, 0x7b, 0x7a, 0x03, b'a', b'b', b'c'
            ]
        );
        // 空列表 = 空 payload（proto3 空消息序列化为零字节）。
        assert!(encode_diff_snapshot_list(&[]).is_empty());
    }
}
