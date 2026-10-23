//! BinaryMD5 archive export for markers.
//!
//! Mirrors Java `MarkerDocController` / `MarkerDaoImpl.refreshMarkerBinaryList`.
//! Markers are grouped by `hidden_flag`; the **normal** group (flag 0) is
//! further split into pages of 3000 by `marker.id / 3000`. All other flag
//! groups are a single page (index 0).
//!
//! A result-level cache avoids re-scanning the whole table on every request.

use anyhow::Result;
use std::collections::BTreeMap;

use _database::{
    DatabaseConnectionMap,
    models::{
        item::item as item_model,
        marker::{
            marker as marker_model, marker_item_link as mil_model, marker_linkage as linkage_model,
        },
    },
};
use _utils::{
    db_operations::SafeEntityTrait, errors::DomainError, jwt::AuthInfo,
    models::wrapper::CommonResponse,
};
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
    db: &DatabaseConnectionMap,
    auth: AuthInfo,
    _payload: serde_json::Value,
) -> Result<CommonResponse<Vec<BinaryMd5Vo>>> {
    // 可见性（Java listMarkerBinaryMD5）：低等级用户拿不到高等级分组的
    // md5，隐藏/测试服页对普通用户如同不存在。
    let allowed = _utils::types::allowed_hidden_flags(auth.info.role_id);
    let entries = marker_result(db).await?;
    Ok(CommonResponse::new(Ok(entries
        .iter()
        .filter(|e| allowed.contains(&entry_flag(&e.key)))
        .map(|e| e.vo.clone())
        .collect())))
}

/// `GET /marker_doc/list_page_bin/{md5}`
pub async fn do_list_page_bin(
    db: &DatabaseConnectionMap,
    auth: AuthInfo,
    md5: String,
) -> Result<Vec<u8>> {
    // 与 md5 清单同口径的角色过滤：知道 md5 也不能取到越权分组。
    let allowed = _utils::types::allowed_hidden_flags(auth.info.role_id);
    let entries = marker_result(db).await?;
    entries
        .iter()
        .find(|e| e.vo.md5 == md5 && allowed.contains(&entry_flag(&e.key)))
        .map(|e| e.bytes.to_vec())
        .ok_or_else(|| DomainError::Business("分页数据未生成或超出获取范围".into()).into())
}

/// Cache key 形如 `marker:{flag}:{page_index}` —— 解析其中的 flag。
fn entry_flag(key: &str) -> i32 {
    key.split(':')
        .nth(1)
        .and_then(|f| f.parse().ok())
        .unwrap_or(-1)
}

/// Compute (and cache) the full marker page set.
async fn marker_result(db: &DatabaseConnectionMap) -> Result<Vec<ResultEntry>> {
    get_result_cached(db, "marker:result".into(), async {
        let db = &db.pg_conn;
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
pub async fn do_list_diff_snapshot(
    db: &DatabaseConnectionMap,
    auth: AuthInfo,
) -> Result<bytes::Bytes> {
    // 可见性（Java HiddenFlagEnum.getFlagListByMask(userDataLevel)）
    let allowed = _utils::types::allowed_hidden_flags(auth.info.role_id);
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
    let mut entries = get_result_cached(db, key, async {
        let db = &db.pg_conn;
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

// ---- 全量点位导出（Java `MarkerDocController#listMarkersByBinary`） ----
//
// Java `MarkerDocService.getMarkerList` → `MarkerDataDaoImpl.buildMarkerListProto`：
// 走 `searchMarker`（areaIdList = 全部地区 → 物品按可见 flag 过滤 → marker_item_link）
// 取点位集合，输出 `MarkerVoList` protobuf（每点位全量 `MarkerVo` + creator/updater
// 用户小表），再 GZIP 压缩。map_front_v3 / map_register_v3（alova + 静态 protobuf
// 解码器）按 `GET /api/marker_doc/list_markers` 拉取全量点位——与 diff snapshot 的
// 区别：这是**全字段**导出且经 GZIP（前端 `Zip.decompress` 后交
// `protobuf.MarkerVoList.decode`）。

/// `SysUserSmallVo { username=1; nickname=2; qq=3; optional phone=4;
/// optional logo=5; optional remark=15; }` 的数据源（sys_user 行，
/// **含软删用户** —— Java `selectUserWithDelete` 契约：creator/updater 可能
/// 已被删除，但其 username/nickname 仍要能被前端解析展示）。
pub struct SysUserSmallInfo {
    pub username: String,
    pub nickname: String,
    pub qq: String,
    pub phone: Option<String>,
    pub logo: Option<String>,
    pub remark: Option<String>,
}

/// LEN 字段（string / bytes / message / packed）：tag + varint 长度 + 内容。
fn put_len_field(out: &mut Vec<u8>, field: u64, bytes: &[u8]) {
    put_varint(out, tag(field, 2));
    put_varint(out, bytes.len() as u64);
    out.extend_from_slice(bytes);
}

/// 隐式存在语义的 string 字段：空串（proto3 默认值）不落 wire，与 Java
/// protobuf `nullToEmpty` + builder 的序列化字节一致。
fn put_string_field(out: &mut Vec<u8>, field: u64, s: &str) {
    if !s.is_empty() {
        put_len_field(out, field, s.as_bytes());
    }
}

/// 隐式存在语义的 varint 字段：0（默认值）不落 wire。
fn put_u64_field(out: &mut Vec<u8>, field: u64, v: u64) {
    if v != 0 {
        put_varint(out, tag(field, 0));
        put_varint(out, v);
    }
}

/// 隐式存在语义的 fixed32 字段（float）：0.0（默认值）不落 wire。
fn put_f32_field(out: &mut Vec<u8>, field: u64, v: f32) {
    if v != 0.0 {
        put_varint(out, tag(field, 5));
        out.extend_from_slice(&v.to_le_bytes());
    }
}

/// 显式存在语义（proto3 `optional`）的 bool 字段：只要调用方给出值就落
/// wire（Java builder `setXxx(false)` 同样序列化）。
fn put_bool_present(out: &mut Vec<u8>, field: u64, v: bool) {
    put_varint(out, tag(field, 0));
    out.push(u8::from(v));
}

/// 显式存在语义（proto3 `optional`）的 varint 字段：值即使是 0 也落 wire。
fn put_u64_present(out: &mut Vec<u8>, field: u64, v: u64) {
    put_varint(out, tag(field, 0));
    put_varint(out, v);
}

/// `MarkerItemLinkVo { item_id=1; icon_id=2; count=10; }`。
fn encode_item_link(item: &_utils::models::marker::MarkerItemLinkVo) -> Vec<u8> {
    let mut msg = Vec::new();
    put_u64_field(&mut msg, 1, item.item_id as u64);
    put_u64_field(&mut msg, 2, item.icon_id as u64);
    put_u64_field(&mut msg, 10, item.count as u64);
    msg
}

/// `MarkerExtra`（JSON 形状 = Java `MarkerExtraVo` 的 wire 命名）：
/// `{ "underground": {"is_underground", "is_global", "region_levels"},
///    "iconOverride": {"id", "minZoom", "maxZoom"},
///    "1_6_island": [str], "2_8_island": {"island_name", "island_state"} }`。
///
/// 消息字段为显式存在：Java 对 extra 恒调 `setExtra`（null 时 set default
/// instance），因此 field 400 始终落 wire（可为零长度 payload）；子消息仅在
/// JSON 里有对应键时落 wire。
fn encode_extra(extra: Option<&serde_json::Value>) -> Vec<u8> {
    let empty = serde_json::Value::Null;
    let extra = extra.unwrap_or(&empty);
    let obj = extra.as_object();

    let mut msg = Vec::new();
    if let Some(ug) = obj
        .and_then(|o| o.get("underground"))
        .and_then(|v| v.as_object())
    {
        let mut inner = Vec::new();
        if ug.get("is_underground") == Some(&serde_json::Value::Bool(true)) {
            put_bool_present(&mut inner, 1, true);
        }
        if let Some(g) = ug.get("is_global") {
            // optional bool：显式存在，false 也序列化（Java setIsGlobal(false)）。
            if let Some(b) = g.as_bool() {
                put_bool_present(&mut inner, 2, b);
            }
        }
        if let Some(levels) = ug.get("region_levels").and_then(|v| v.as_array()) {
            for lv in levels {
                if let Some(s) = lv.as_str() {
                    put_string_field(&mut inner, 9, s);
                }
            }
        }
        put_len_field(&mut msg, 1, &inner);
    }
    if let Some(io) = obj
        .and_then(|o| o.get("iconOverride"))
        .and_then(|v| v.as_object())
    {
        let mut inner = Vec::new();
        put_u64_field(
            &mut inner,
            1,
            io.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as u64,
        );
        put_f32_field(
            &mut inner,
            2,
            io.get("minZoom").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
        );
        put_f32_field(
            &mut inner,
            3,
            io.get("maxZoom").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
        );
        put_len_field(&mut msg, 2, &inner);
    }
    if let Some(islands) = obj
        .and_then(|o| o.get("1_6_island"))
        .and_then(|v| v.as_array())
    {
        for island in islands {
            if let Some(s) = island.as_str() {
                put_string_field(&mut msg, 100, s);
            }
        }
    }
    if let Some(v28) = obj.and_then(|o| o.get("2_8_island")) {
        let mut inner = Vec::new();
        put_string_field(
            &mut inner,
            1,
            v28.get("island_name")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
        );
        if let Some(states) = v28.get("island_state").and_then(|v| v.as_array()) {
            for st in states {
                if let Some(s) = st.as_str() {
                    put_string_field(&mut inner, 2, s);
                }
            }
        }
        put_len_field(&mut msg, 101, &inner);
    }
    msg
}

/// `MarkerVo` 单条消息（字段号见前端 `MarkerVo.proto` = Java 侧同源 proto）。
fn encode_full_marker(v: &_utils::models::marker::MarkerVO) -> Vec<u8> {
    let mut msg = Vec::new();
    put_u64_field(&mut msg, 1, v.version as u64);
    put_u64_field(&mut msg, 2, v.id as u64);
    if let Some(c) = v.creator_id {
        put_u64_field(&mut msg, 3, c as u64);
    }
    // optional uint64：Java 仅在 Date 非空时 set（显式存在，0 也落 wire）。
    put_u64_present(&mut msg, 4, v.create_time as u64);
    if let Some(u) = v.updater_id {
        put_u64_field(&mut msg, 5, u as u64);
    }
    if let Some(t) = v.update_time {
        put_u64_present(&mut msg, 6, t as u64);
    }
    put_string_field(&mut msg, 8, v.marker_title.as_deref().unwrap_or(""));
    // Java blankToDefault(position, "0,0")：空坐标补默认值。
    let position = if v.position.trim().is_empty() {
        "0,0"
    } else {
        v.position.as_str()
    };
    put_string_field(&mut msg, 9, position);
    put_string_field(&mut msg, 10, v.content.as_deref().unwrap_or(""));
    put_string_field(&mut msg, 11, v.picture.as_deref().unwrap_or(""));
    put_string_field(&mut msg, 12, v.video_path.as_deref().unwrap_or(""));
    put_u64_field(&mut msg, 13, v.refresh_time as u64);
    put_u64_field(&mut msg, 14, v.hidden_flag as i32 as u64);
    for item in &v.item_list {
        let link = encode_item_link(item);
        put_len_field(&mut msg, 15, &link);
    }
    // optional uint64（DB 列非空，Java 恒 set；0 也落 wire 以保显式存在语义）。
    put_u64_present(&mut msg, 100, v.marker_creator_id as u64);
    if let Some(p) = v.picture_creator_id {
        put_u64_present(&mut msg, 101, p as u64);
    }
    put_string_field(&mut msg, 200, v.marker_stamp.as_deref().unwrap_or(""));
    // 消息字段恒落 wire（Java setExtra(defaultInstance) 亦序列化零长度消息）。
    let extra = encode_extra(v.extra.as_ref());
    put_len_field(&mut msg, 400, &extra);
    put_string_field(&mut msg, 500, v.linkage_id.as_deref().unwrap_or(""));
    msg
}

/// `SysUserSmallVo` 单条消息。
fn encode_sys_user_small(u: &SysUserSmallInfo) -> Vec<u8> {
    let mut msg = Vec::new();
    put_string_field(&mut msg, 1, &u.username);
    put_string_field(&mut msg, 2, &u.nickname);
    put_string_field(&mut msg, 3, &u.qq);
    if let Some(p) = u.phone.as_deref() {
        put_len_field(&mut msg, 4, p.as_bytes());
    }
    if let Some(l) = u.logo.as_deref() {
        put_len_field(&mut msg, 5, l.as_bytes());
    }
    if let Some(r) = u.remark.as_deref() {
        put_len_field(&mut msg, 15, r.as_bytes());
    }
    msg
}

/// `MarkerVoList { repeated MarkerVo markers = 1;
/// map<uint64, SysUserSmallVo> users = 2; }`。map 条目编码为
/// `{ key=1 varint, value=2 message }` 子消息。
pub fn encode_marker_vo_list(
    markers: &[_utils::models::marker::MarkerVO],
    users: &std::collections::BTreeMap<u64, SysUserSmallInfo>,
) -> Vec<u8> {
    let mut out = Vec::new();
    for m in markers {
        let msg = encode_full_marker(m);
        put_len_field(&mut out, 1, &msg);
    }
    for (uid, user) in users {
        let mut entry = Vec::new();
        put_u64_present(&mut entry, 1, *uid);
        let value = encode_sys_user_small(user);
        put_len_field(&mut entry, 2, &value);
        put_len_field(&mut out, 2, &entry);
    }
    out
}

/// 与 `visible_markers_query` 同形状的全列查询：join 语义、软删与可见性
/// 过滤、DISTINCT 去重、按 id 稳定排序完全一致，只是带出 marker 全部列
/// （list_markers 需要全字段，diff snapshot 只要 version/id）。
fn visible_markers_full_query(allowed: &[i32]) -> sea_orm::Select<marker_model::Entity> {
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
        // 全列去重只能用 DISTINCT ON (id)：marker.extra 是 json 列，PG 的
        // 普通 DISTINCT 需要整行等值算子，对 json 直接报错
        // （"could not identify an equality operator for type json"）。
        // 一点位多链的重复行按 id 去重，排序仍按 id 稳定。
        .distinct_on([marker_model::Column::Id])
        .order_by_asc(marker_model::Column::Id)
}
/// GZIP 压缩（Java `CompressUtils.compress`；前端 `Zip.decompress` 解包）。
fn gzip(bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
    use std::io::Write;
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(bytes)?;
    Ok(encoder.finish()?)
}

/// `GET /marker_doc/list_markers`：全量可见点位（GZIP 的 `MarkerVoList`
/// protobuf 字节）。
///
/// 高频只读接口：与 diff snapshot 相同的两级缓存（moka + Redis），缓存键
/// 按可见 flag 集区分；marker / linkage / item 写路径的
/// `invalidate_doc_cache()` 整体失效本键。
pub async fn do_list_markers(db: &DatabaseConnectionMap, auth: AuthInfo) -> Result<bytes::Bytes> {
    let allowed = _utils::types::allowed_hidden_flags(auth.info.role_id);
    let mut flags = allowed.clone();
    flags.sort_unstable();
    let key = format!(
        "marker:list_markers:{}",
        flags
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    let mut entries = get_result_cached(db, key, async {
        let db = &db.pg_conn;
        let bytes = list_markers_bytes(db, &allowed).await?;
        let digest = md5::compute(&bytes);
        Ok(vec![ResultEntry {
            key: "marker:list_markers".into(),
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

/// 全量点位字节组装：可见点位（含 item_list / linkage_id）→ `MarkerVoList`
/// protobuf → GZIP。users 小表取 creator/updater 并集（含软删用户）。
pub async fn list_markers_bytes(
    db: &sea_orm::DatabaseConnection,
    allowed: &[i32],
) -> Result<Vec<u8>> {
    let markers: Vec<marker_model::Model> = visible_markers_full_query(allowed).all(db).await?;
    let ids: Vec<i64> = markers.iter().map(|m| m.id).collect();
    let item_map = super::marker::marker_item_map(db, &ids).await?;
    let linkage_map = super::marker::marker_linkage_map(db, &ids).await?;

    let vos: Vec<_> = markers
        .iter()
        .map(|m| super::marker::model_to_vo_doc(m, &item_map, Some(&linkage_map)))
        .collect();
    // users 小表：creator + updater 并集。不用 find_safety —— Java
    // selectUserWithDelete 含软删用户（点位作者被删后昵称仍需可解析）。
    let mut user_ids: Vec<i64> = vos
        .iter()
        .flat_map(|v| [v.creator_id, v.updater_id])
        .flatten()
        .collect();
    user_ids.sort_unstable();
    user_ids.dedup();
    let mut users: std::collections::BTreeMap<u64, SysUserSmallInfo> =
        std::collections::BTreeMap::new();
    if !user_ids.is_empty() {
        use _database::models::system::sys_user as sys_user_model;
        let rows = sys_user_model::Entity::find()
            .filter(sys_user_model::Column::Id.is_in(user_ids))
            .all(db)
            .await?;
        for u in rows {
            users.insert(
                u.id as u64,
                SysUserSmallInfo {
                    username: u.username,
                    nickname: u.nickname.unwrap_or_default(),
                    qq: u.qq.unwrap_or_default(),
                    phone: u.phone,
                    logo: u.logo,
                    remark: u.remark,
                },
            );
        }
    }

    gzip(&encode_marker_vo_list(&vos, &users))
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

    fn sample_vo(id: i64, title: Option<&str>) -> _utils::models::marker::MarkerVO {
        _utils::models::marker::MarkerVO {
            version: 3,
            id,
            create_time: 1_700_000_000_000.0,
            update_time: None,
            creator_id: Some(7),
            updater_id: None,
            del_flag: false,
            marker_stamp: None,
            marker_title: title.map(str::to_string),
            position: "123.4,56.7".into(),
            content: None,
            picture: None,
            marker_creator_id: 7,
            picture_creator_id: None,
            video_path: None,
            refresh_time: 0,
            hidden_flag: _utils::types::HiddenFlag::Visible,
            extra: None,
            item_list: vec![_utils::models::marker::MarkerItemLinkVo {
                item_id: 42,
                count: 2,
                icon_tag: None,
                icon_id: 9,
            }],
            linkage_id: Some("g1".into()),
        }
    }

    fn sample_user() -> SysUserSmallInfo {
        SysUserSmallInfo {
            username: "alice".into(),
            nickname: "Alice".into(),
            qq: "10001".into(),
            phone: None,
            logo: None,
            remark: None,
        }
    }

    #[test]
    fn full_marker_wire_matches_hand_computed_prefix() {
        // {version:3, id:11, creator:7}: 08 03 | 10 0b | 18 07 —— 随后
        // create_time（f4 tag=32，显式存在）。标题 "t" 在 f8：42 01 74。
        let vo = sample_vo(11, Some("t"));
        let bytes = encode_full_marker(&vo);
        assert_eq!(&bytes[..6], &[0x08, 0x03, 0x10, 0x0b, 0x18, 0x07]);
        assert_eq!(&bytes[13..16], &[0x42, 0x01, b't']);
        // extra（f400）恒落 wire：tag varint(3202) = 82 19 + 长度 0。
        assert!(
            bytes.windows(3).any(|w| w == [0x82, 0x19, 0x00]),
            "extra field must be present: {bytes:?}"
        );
        // 末尾是 linkage_id（f500）：tag varint(4002) = a2 1f，LEN 2，"g1"。
        assert_eq!(&bytes[bytes.len() - 5..], &[0xa2, 0x1f, 0x02, b'g', b'1']);
    }

    #[test]
    fn full_marker_omits_default_scalars_and_empty_strings() {
        // 全默认值 VO：隐式存在标量（version/id/refresh/hidden）与空串
        // 全部不落 wire，只剩显式存在的 create_time(4) / marker_creator_id(100)
        // / extra(400)。
        let mut vo = sample_vo(0, None);
        vo.version = 0;
        vo.creator_id = None;
        vo.create_time = 0.0;
        vo.marker_creator_id = 0;
        vo.position = "".into();
        vo.item_list.clear();
        vo.linkage_id = None;
        let bytes = encode_full_marker(&vo);
        assert_eq!(
            bytes,
            vec![
                0x20, 0x00, // f4 create_time（显式存在，0 也落 wire）
                0x4a, 0x03, b'0', b',', b'0', // f9 position 空串补默认 "0,0"
                0xa0, 0x06, 0x00, // f100 marker_creator_id（显式存在）
                0x82, 0x19, 0x00, // f400 extra（消息恒落 wire）
            ]
        );
    }

    #[test]
    fn full_marker_blank_position_defaults_to_zero_zero() {
        // Java blankToDefault(position, "0,0")。
        let mut vo = sample_vo(1, None);
        vo.position = "   ".into();
        let bytes = encode_full_marker(&vo);
        assert!(
            bytes
                .windows(5)
                .any(|w| w == [0x4a, 0x03, b'0', b',', b'0']),
            "field 9 must carry the 0,0 default: {bytes:?}"
        );
    }

    #[test]
    fn item_link_encodes_count_and_ids() {
        let link = _utils::models::marker::MarkerItemLinkVo {
            item_id: 42,
            count: 2,
            icon_tag: None,
            icon_id: 9,
        };
        // {item_id:42, icon_id:9, count:2}：08 2a | 10 09 | 50 02。
        assert_eq!(
            encode_item_link(&link),
            vec![0x08, 0x2a, 0x10, 0x09, 0x50, 0x02]
        );
    }

    #[test]
    fn extra_encodes_all_four_subobjects() {
        let extra = serde_json::json!({
            "underground": {"is_underground": true, "is_global": false, "region_levels": ["A", "B"]},
            "iconOverride": {"id": 5, "minZoom": 1.5, "maxZoom": 0.0},
            "1_6_island": ["x"],
            "2_8_island": {"island_name": "n", "island_state": ["s1"]}
        });
        let bytes = encode_extra(Some(&extra));
        // underground f1(0x0a)、iconOverride f2(0x12)、1_6_island
        // f100(0x82 0x06)、2_8_island f101(0x8a 0x06)。
        for t in [vec![0x0a], vec![0x12], vec![0xa2, 0x06], vec![0xaa, 0x06]] {
            assert!(
                bytes.windows(t.len()).any(|w| w == t),
                "missing subfield tag {t:?} in {bytes:?}"
            );
        }
        // is_global=false 显式存在（optional bool）→ underground 内含
        // `10 00`（field2 wire0 值 0）。
        assert!(bytes.windows(2).any(|w| w == [0x10, 0x00]));
    }

    #[test]
    fn sys_user_small_sets_optional_fields_only_when_present() {
        let u = SysUserSmallInfo {
            username: "u".into(),
            nickname: "n".into(),
            qq: "q".into(),
            phone: Some("p".into()),
            logo: None,
            remark: None,
        };
        let bytes = encode_sys_user_small(&u);
        assert_eq!(bytes[0], 0x0a);
        assert!(bytes.windows(2).any(|w| w == [0x22, 0x01])); // field4 LEN len1
        assert!(!bytes.windows(2).any(|w| w[0] == 0x2a)); // field5 absent
        assert!(!bytes.windows(2).any(|w| w[0] == 0x7a)); // field15 absent
    }

    #[test]
    fn marker_vo_list_wraps_markers_and_users_map() {
        let vos = vec![sample_vo(11, None)];
        let mut users = std::collections::BTreeMap::new();
        users.insert(7_u64, sample_user());
        let bytes = encode_marker_vo_list(&vos, &users);
        // 首字段是 markers f1（0x0a）；map 条目为 f2（0x12）子消息
        // {key f1 varint, value f2 LEN}。
        assert_eq!(bytes[0], 0x0a);
        let idx = bytes
            .windows(2)
            .position(|w| w[0] == 0x12)
            .expect("map entry");
        let entry = &bytes[idx + 1..];
        assert_eq!(entry[1], 0x08); // key tag
        assert_eq!(entry[2], 0x07); // user id = 7
    }

    #[test]
    fn gzip_roundtrip_of_encoded_list() {
        let vos = vec![sample_vo(1, Some("t")), sample_vo(2, None)];
        let raw = encode_marker_vo_list(&vos, &std::collections::BTreeMap::new());
        let compressed = gzip(&raw).expect("gzip");
        assert_eq!(&compressed[..2], &[0x1f, 0x8b]); // GZIP magic
        use std::io::Read;
        let mut dec = flate2::read::GzDecoder::new(&compressed[..]);
        let mut out = Vec::new();
        dec.read_to_end(&mut out).expect("gunzip");
        assert_eq!(out, raw);
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
