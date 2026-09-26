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
        marker::{
            marker as marker_model, marker_item_link as mil_model, marker_linkage as linkage_model,
        },
    },
};
use _utils::{db_operations::SafeEntityTrait, jwt::AuthInfo, models::wrapper::CommonResponse};
use sea_orm::{ColumnTrait, JoinType, QueryFilter, QueryOrder, QuerySelect, prelude::*};

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
        .map(|e| e.bytes.to_vec())
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
                            bytes: compressed.into(),
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
                        bytes: compressed.into(),
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
pub struct DiffSnapshot {
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
pub fn encode_diff_snapshot_list(snapshots: &[DiffSnapshot]) -> Vec<u8> {
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
///
/// 高频接口：结果按可见 flag 集走 `get_result_cached`（本进程 moka →
/// Redis 跨副本共享 → 计算）。任一副本冷启动或写后失效重建时，先到者
/// 计算一次并写入 Redis，其余副本直接取字节，避免各自全量重扫数据库。
/// marker / linkage 写路径的 invalidate_doc_cache() 会整体失效本键。
/// 返回 [`bytes::Bytes`]，缓存命中时整条服务链路零拷贝。
pub async fn do_list_diff_snapshot(auth: AuthInfo) -> Result<bytes::Bytes> {
    // 可见性（Java HiddenFlagEnum.getFlagListByMask(userDataLevel)）
    let allowed = _utils::types::allowed_hidden_flags(auth.info.role_id);
    let db = &DB_CONN.wait().pg_conn;

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
    let mut entries = get_result_cached(key, async {
        let bytes = diff_snapshot_bytes(db, &allowed).await?;
        let digest = md5::compute(&bytes);
        Ok(vec![ResultEntry {
            key: "marker:diff_snapshot".into(),
            vo: BinaryMd5Vo {
                md5: format!("{:x}", digest),
                time: chrono::Utc::now().timestamp_millis(),
            },
            bytes: bytes.into(),
        }])
    })
    .await?;
    Ok(entries.pop().map(|e| e.bytes).unwrap_or_default())
}

/// 可见点位集合查询（`marker ⋈ marker_item_link ⋈ item`），供
/// `diff_snapshot_bytes` 与其 SQL 形状单测共用。
fn visible_markers_query(allowed: &[i32]) -> sea_orm::Select<marker_model::Entity> {
    marker_model::Entity::find()
        .join_rev(
            JoinType::InnerJoin,
            mil_model::Entity::belongs_to(marker_model::Entity)
                .from(mil_model::Column::MarkerId)
                .to(marker_model::Column::Id)
                .into(),
        )
        .join(
            JoinType::InnerJoin,
            mil_model::Entity::belongs_to(item_model::Entity)
                .from(mil_model::Column::ItemId)
                .to(item_model::Column::Id)
                .into(),
        )
        .filter(marker_model::Column::DelFlag.eq(false))
        .filter(marker_model::Column::HiddenFlag.is_in(allowed.to_vec()))
        .filter(mil_model::Column::DelFlag.eq(false))
        .filter(item_model::Column::DelFlag.eq(false))
        .filter(item_model::Column::HiddenFlag.is_in(allowed.to_vec()))
        .select_only()
        .column(marker_model::Column::Version)
        .column(marker_model::Column::Id)
        .distinct()
        .order_by_asc(marker_model::Column::Id)
}

/// Java `searchMarkerId`（areaIdList = 全部地区）+ `listMarkerById`：
/// 物品按可见 flag 过滤 → marker_item_link → 点位（del_flag=false 且
/// 可见 flag）→ (version, id) 集合，再附上每点位首个关联组 ID。
///
/// 一条 `marker ⋈ marker_item_link ⋈ item` 集合查询取代旧实现
/// 「物品 ids → 分块 IN 关联 → 分块 IN 点位」的三段式：后者是
/// ~3×⌈N/1000⌉ 次数据库往返、每块数千字节的字面量 IN 列表，100k 点位
/// 规模下约两百次往返。语义等价：点位须同时可见、且经未删除的关联
/// 链接到至少一个可见物品（`DISTINCT` 去掉一点多链的重复）。
/// `ORDER BY id` 让快照字节跨副本稳定。
pub async fn diff_snapshot_bytes(
    db: &sea_orm::DatabaseConnection,
    allowed: &[i32],
) -> Result<Vec<u8>> {
    let markers: Vec<(i64, i64)> = visible_markers_query(allowed)
        .into_tuple::<(i64, i64)>()
        .all(db)
        .await?;

    // 连线组表远小于点位表：三列全量顺序扫描一次，比按点位 id 分块
    // OR-IN（又一轮 ⌈N/1000⌉ 次往返）便宜，再在内存里过滤可见集。
    let linkage_pairs: Vec<(i64, i64, String)> = linkage_model::Entity::find_safety()
        .select_only()
        .column(linkage_model::Column::FromId)
        .column(linkage_model::Column::ToId)
        .column(linkage_model::Column::GroupId)
        .into_tuple::<(i64, i64, String)>()
        .all(db)
        .await?;

    Ok(encode_diff_snapshot_list(&assemble_snapshots(
        markers,
        &linkage_pairs,
    )))
}

/// 把 (version, id) 行集合与连线组三元组 (from_id, to_id, group_id)
/// 拼装成快照列表。连线命中多组时取第一条（与 `marker_linkage_map`
/// 的首插即定语义一致）；两端都不在可见点位集内的连线行直接丢弃。
pub fn assemble_snapshots(
    markers: Vec<(i64, i64)>,
    linkage_pairs: &[(i64, i64, String)],
) -> Vec<DiffSnapshot> {
    let visible: std::collections::HashSet<i64> = markers.iter().map(|(_, id)| *id).collect();
    let mut linkage: std::collections::HashMap<i64, &String> =
        std::collections::HashMap::with_capacity(linkage_pairs.len().min(visible.len()));
    for (from, to, group_id) in linkage_pairs {
        if visible.contains(from) {
            linkage.entry(*from).or_insert(group_id);
        }
        if visible.contains(to) {
            linkage.entry(*to).or_insert(group_id);
        }
    }
    markers
        .into_iter()
        .map(|(version, id)| DiffSnapshot {
            version: version as u64,
            id: id as u64,
            linkage_id: linkage.get(&id).map(|g| (*g).clone()),
        })
        .collect()
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
    fn diff_snapshot_join_sql_shape_is_stable() {
        use sea_orm::QueryTrait;
        let stmt = visible_markers_query(&[0, 3]).build(sea_orm::DbBackend::Postgres);
        let sql = &stmt.sql;
        // 两跳 JOIN 的 ON 条件 + DISTINCT 去重 + 排序稳定，是本查询的
        // 承重结构；三张表的 del_flag 过滤缺一不可（软删语义）。
        assert!(sql.contains(r#"SELECT DISTINCT "marker"."version", "marker"."id" FROM "marker""#));
        assert!(sql.contains(
            r#"INNER JOIN "marker_item_link" ON "marker_item_link"."marker_id" = "marker"."id""#
        ));
        assert!(sql.contains(r#"INNER JOIN "item" ON "marker_item_link"."item_id" = "item"."id""#));
        assert_eq!(
            sql.matches(r#"."del_flag" = "#).count(),
            3,
            "marker / marker_item_link / item 三表都要过滤软删"
        );
        assert!(sql.contains(r#"ORDER BY "marker"."id" ASC"#));
        // [0, 3] 两个 flag × 两张表 + 三个 del_flag 布尔 = 7 个绑定参数。
        let params = stmt.values.map(|v| v.0.len()).unwrap_or(0);
        assert_eq!(params, 7);
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

    #[test]
    fn assemble_snapshots_joins_linkage_and_keeps_first_group() {
        let markers = vec![(3, 1), (7, 2), (1, 3)];
        let pairs = vec![
            (1, 3, "g1".to_string()),
            (1, 3, "g2".to_string()),
            (99, 1, "g3".to_string()),  // from 不可见，仅 to 命中
            (98, 97, "g4".to_string()), // 两端均不可见 → 丢弃
        ];
        let snaps = assemble_snapshots(markers, &pairs);
        let ids: Vec<u64> = snaps.iter().map(|s| s.id).collect();
        assert_eq!(ids, vec![1, 2, 3]);
        // 多组命中时首插即定（g1 而非 g2）；to_id 命中的点位同样拿到组。
        assert_eq!(snaps[0].linkage_id.as_deref(), Some("g1"));
        assert_eq!(snaps[1].linkage_id, None);
        assert_eq!(snaps[2].linkage_id.as_deref(), Some("g1"));
    }

    #[test]
    fn assemble_snapshots_empty_inputs_yield_empty_list() {
        assert!(assemble_snapshots(Vec::new(), &[]).is_empty());
        // 无任何连线时全部点位 linkage_id 缺省。
        let snaps = assemble_snapshots(vec![(1, 5)], &[(9, 8, "g".to_string())]);
        assert_eq!(snaps.len(), 1);
        assert_eq!(snaps[0].linkage_id, None);
    }
}
