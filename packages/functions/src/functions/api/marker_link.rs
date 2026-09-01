use anyhow::{Result, anyhow};

use sea_orm::{
    ActiveValue::{NotSet, Set},
    QueryFilter, QuerySelect,
    prelude::*,
};

use _database::{
    DB_CONN,
    models::marker::{marker as marker_model, marker_linkage as linkage_model},
};
use _utils::types::MarkerLinkageLinkAction;
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        marker_link::{
            MarkerLinkDeleteRequest, MarkerLinkGraphRequest, MarkerLinkListRequest, MarkerLinkVO,
            MarkerLinkage, MarkerLinkagePathEdge,
        },
        wrapper::CommonResponse,
    },
};

use std::collections::{BTreeMap, HashMap, HashSet};

/// Java `MarkerLinkageVo` wire 值（`LinkActionEnum`）。
fn action_str(action: MarkerLinkageLinkAction) -> String {
    serde_json::to_value(action)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| "TRIGGER".into())
}

/// Java `MarkerLinkageDataHelper.reverseLinkageIds`：写入侧按 from<to 规范化
/// （`link_reverse=true` 记录「存储方向与逻辑方向相反」），读取侧据此交换回
/// from/to 并复位标记 —— 前端拿到的永远是逻辑方向。
pub(super) fn reverse_linkage_vos(vos: &mut [MarkerLinkVO]) {
    for vo in vos.iter_mut() {
        if vo.link_reverse.unwrap_or(false) {
            std::mem::swap(&mut vo.from_id, &mut vo.to_id);
            vo.link_reverse = Some(false);
        }
    }
}

/// Java `MarkerLinkageDataHelper.getPathMarkerIdsFromList`：收集路径边引用的
/// 全部点位 ID（id1/id2 > 0，去重）。
pub(super) fn path_marker_ids(vos: &[MarkerLinkVO]) -> Vec<i64> {
    let mut set: HashSet<i64> = HashSet::new();
    for vo in vos {
        for edge in vo.path.iter().flatten().flatten() {
            if edge.id1.unwrap_or(0) > 0 {
                set.insert(edge.id1.unwrap());
            }
            if edge.id2.unwrap_or(0) > 0 {
                set.insert(edge.id2.unwrap());
            }
        }
    }
    set.into_iter().collect()
}

/// Java `MarkerLinkageHelperService.getPathCoords`：解析 marker.position
///（"{x},{y}"）为坐标表。
pub(super) async fn path_marker_coords(
    db: &sea_orm::DatabaseConnection,
    ids: &[i64],
) -> Result<HashMap<i64, (f64, f64)>> {
    let mut coords: HashMap<i64, (f64, f64)> = HashMap::new();
    if ids.is_empty() {
        return Ok(coords);
    }
    for chunk in ids.chunks(1000) {
        for (id, position) in marker_model::Entity::find()
            .filter(marker_model::Column::Id.is_in(chunk))
            .select_only()
            .column(marker_model::Column::Id)
            .column(marker_model::Column::Position)
            .into_tuple::<(i64, String)>()
            .all(db)
            .await?
        {
            let Some((x, y)) = position.split_once(',') else {
                continue;
            };
            let (Ok(x), Ok(y)) = (x.trim().parse::<f64>(), y.trim().parse::<f64>()) else {
                continue;
            };
            coords.insert(id, (x, y));
        }
    }
    Ok(coords)
}

/// Java `MarkerLinkageDataHelper.patchPathMarkerCoordsInList`：按 id1/id2 回填
/// x1/y1/x2/y2。
pub(super) fn patch_path_coords(vos: &mut [MarkerLinkVO], coords: &HashMap<i64, (f64, f64)>) {
    for vo in vos.iter_mut() {
        for edge in vo.path.iter_mut().flatten().flatten() {
            if let Some(&(x, y)) = coords.get(&edge.id1.unwrap_or(0)) {
                edge.x1 = Some(x);
                edge.y1 = Some(y);
            }
            if let Some(&(x, y)) = coords.get(&edge.id2.unwrap_or(0)) {
                edge.x2 = Some(x);
                edge.y2 = Some(y);
            }
        }
    }
}

/// 载入分组关联并完成 Java 读侧两步变换（reverse + 坐标回填）。
async fn load_group_vos(
    db: &sea_orm::DatabaseConnection,
    group_ids: &[String],
) -> Result<Vec<MarkerLinkVO>> {
    let mut vos: Vec<MarkerLinkVO> = Vec::new();
    for chunk in group_ids.chunks(1000) {
        for it in linkage_model::Entity::find_safety()
            .filter(linkage_model::Column::GroupId.is_in(chunk))
            .all(db)
            .await?
        {
            vos.push(model_to_vo(it));
        }
    }
    reverse_linkage_vos(&mut vos);
    let path_ids = path_marker_ids(&vos);
    let coords = path_marker_coords(db, &path_ids).await?;
    patch_path_coords(&mut vos, &coords);
    Ok(vos)
}

/// 将数据库模型转换为前端 VO（字段与前端 `MarkerLinkageVo` 对齐）
pub(crate) fn model_to_vo(it: linkage_model::Model) -> MarkerLinkVO {
    MarkerLinkVO {
        id: it.id,
        version: it.version,
        group_id: Some(it.group_id),
        from_id: it.from_id,
        to_id: it.to_id,
        link_action: Some(it.link_action),
        link_reverse: Some(it.link_reverse),
        path: it.path.and_then(|j| serde_json::from_value(j).ok()),
        creator_id: it.creator_id,
        updater_id: it.updater_id,
        // DB 更新时间为空时回退创建时间，保证前端增量判断（updateTime）可用
        update_time: it
            .update_time
            .or(Some(it.create_time))
            .map(|t| t.and_utc().timestamp_millis() as f64),
    }
}

/// Java `MarkerLinkageDataHelper.getIdHash` 的无序对键：pair → (min, max)。
fn pair_key(a: i64, b: i64) -> (i64, i64) {
    (a.min(b), a.max(b))
}

/// do_link 的待落库行：既有关联行（含软删）或新建行，附提交值与删除标记。
struct PendingLink {
    existing: Option<linkage_model::Model>,
    from_id: i64,
    to_id: i64,
    link_reverse: bool,
    link_action: Option<MarkerLinkageLinkAction>,
    path: Option<serde_json::Value>,
    del: bool,
}

/// 建立关联（Java `MarkerLinkageService.linkMarker` + `patchLinkSearchMap`）：
/// - 存储方向规范化 from<to，`link_reverse` 记录逻辑反向；入参携带的
///   `linkReverse` 一律忽略，方向语义只看 fromId -> toId；
/// - 与提交端点相关的**全部**既有关联（含软删）进入置换集：未再次提交的
///   行被软删（缺席清理），再次提交的行复用/复活并划入新组；
/// - 同一无序对天然去重（同对只保留一行）。
pub async fn do_link(
    auth: AuthInfo,
    payload: Vec<MarkerLinkage>,
) -> Result<CommonResponse<String>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;

    // 校验（Java checkLinkList 同文案）：非空、端点合法、禁止自关联。
    if payload.is_empty() {
        return Err(anyhow!("关联数据不可为空"));
    }
    for p in &payload {
        if p.from_id <= 0 || p.to_id <= 0 {
            return Err(anyhow!("无效的关联节点ID"));
        }
        if p.from_id == p.to_id {
            return Err(anyhow!("不能将点位关联到自身"));
        }
    }

    let group_id = uuid::Uuid::new_v4().simple().to_string();

    // 变更集（Java LinkChangeVo）：先收提交端点，再补既有关联行的组与端点
    let mut affected_groups: Vec<String> = Vec::new();
    let mut affected_markers: HashSet<i64> = HashSet::new();
    for p in &payload {
        affected_markers.insert(p.from_id);
        affected_markers.insert(p.to_id);
    }
    let endpoint_vec: Vec<i64> = affected_markers.iter().copied().collect();

    // 与提交端点相关的全部既有关联（含软删；Java getRelatedLinkageList(ids, true)）
    let mut related: Vec<linkage_model::Model> = Vec::new();
    for chunk in endpoint_vec.chunks(1000) {
        related.extend(
            linkage_model::Entity::find()
                .filter(
                    sea_orm::Condition::any()
                        .add(linkage_model::Column::FromId.is_in(chunk))
                        .add(linkage_model::Column::ToId.is_in(chunk)),
                )
                .all(db)
                .await?,
        );
    }
    for r in &related {
        if !r.group_id.is_empty() && !affected_groups.contains(&r.group_id) {
            affected_groups.push(r.group_id.clone());
        }
        affected_markers.insert(r.from_id);
        affected_markers.insert(r.to_id);
    }

    // Java getLinkSearchMap + patchLinkSearchMap：无序对 → 行；
    // 先全部标记删除，再激活提交的无序对（复用既有行或新建）。
    let mut index: HashMap<(i64, i64), usize> = HashMap::new();
    let mut pending: Vec<PendingLink> = Vec::new();
    for r in related {
        let key = pair_key(r.from_id, r.to_id);
        index.insert(key, pending.len());
        pending.push(PendingLink {
            existing: Some(r),
            from_id: 0,
            to_id: 0,
            link_reverse: false,
            link_action: None,
            path: None,
            del: true,
        });
    }
    for p in &payload {
        // 存储方向规范化：from>to 时交换并置反向标记
        let (from_id, to_id, link_reverse) = if p.from_id > p.to_id {
            (p.to_id, p.from_id, true)
        } else {
            (p.from_id, p.to_id, false)
        };
        let key = pair_key(p.from_id, p.to_id);
        // 业务默认值：path = []（客户端缺省路径时落空数组而非 NULL）
        let path_json = Some(
            p.path
                .as_ref()
                .and_then(|v| serde_json::to_value(v).ok())
                .unwrap_or_else(|| serde_json::json!([])),
        );
        if let Some(&i) = index.get(&key) {
            pending[i].del = false;
            pending[i].from_id = from_id;
            pending[i].to_id = to_id;
            pending[i].link_reverse = link_reverse;
            pending[i].link_action = p.link_action;
            pending[i].path = path_json;
        } else {
            index.insert(key, pending.len());
            pending.push(PendingLink {
                existing: None,
                from_id,
                to_id,
                link_reverse,
                link_action: p.link_action,
                path: path_json,
                del: false,
            });
        }
    }

    // 落库（Java saveOrUpdateBatch：复活 → 插入/更新 → 软删缺席行）
    for pl in pending {
        if pl.del {
            if let Some(m) = pl.existing {
                let mut am: linkage_model::ActiveModel = m.into();
                am.del_flag = Set(true);
                // 审计字段：软删也是修改，设置 update 组
                am.updater_id = Set(Some(auth.info.id));
                linkage_model::Entity::update_safety(am)?.exec(db).await?;
            }
            continue;
        }
        let action = pl.link_action.unwrap_or(MarkerLinkageLinkAction::Trigger);
        if let Some(m) = pl.existing {
            let mut am: linkage_model::ActiveModel = m.into();
            // 审计字段：修改时设置 update 组（update_time 由 before_save 钩子刷新）
            am.updater_id = Set(Some(auth.info.id));
            am.group_id = Set(group_id.clone());
            am.from_id = Set(pl.from_id);
            am.to_id = Set(pl.to_id);
            am.link_action = Set(action);
            am.link_reverse = Set(pl.link_reverse);
            am.path = Set(pl.path);
            am.del_flag = Set(false);
            linkage_model::Entity::update_safety(am)?.exec(db).await?;
        } else {
            let now = chrono::Utc::now().naive_utc();
            let active = linkage_model::ActiveModel {
                version: Set(0),
                id: NotSet,
                // 审计字段：新增时 create/update 两组全部设置
                create_time: Set(now),
                update_time: Set(Some(now)),
                creator_id: Set(Some(auth.info.id)),
                updater_id: Set(Some(auth.info.id)),
                del_flag: Set(false),
                group_id: Set(group_id.clone()),
                from_id: Set(pl.from_id),
                to_id: Set(pl.to_id),
                link_action: Set(action),
                link_reverse: Set(pl.link_reverse),
                path: Set(pl.path),
                // 业务默认值：extra = {}
                extra: Set(Some(serde_json::json!({}))),
            };
            active.insert(db).await?;
        }
    }

    if !affected_groups.contains(&group_id) {
        affected_groups.push(group_id.clone());
    }
    let mut markers: Vec<i64> = affected_markers.into_iter().collect();
    markers.sort_unstable();
    super::binary_doc::invalidate_doc_cache().await;
    super::super::ws::ws_broadcast(
        "MarkerLinked",
        serde_json::json!({
            "groups": affected_groups,
            "markers": markers
        }),
    );
    super::super::ws::ws_broadcast_debounced(
        "MarkerBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    super::super::ws::ws_broadcast_debounced(
        "MarkerLinkageBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(CommonResponse::new(Ok(group_id)))
}

pub async fn do_get_list(
    _auth: AuthInfo,
    payload: MarkerLinkListRequest,
) -> Result<CommonResponse<serde_json::Value>> {
    // 与 Java 实现一致：未指定组 ID 时返回空 map
    if payload.group_ids.is_empty() {
        return Ok(CommonResponse::new(Ok(serde_json::json!({}))));
    }
    let db = &DB_CONN.wait().pg_conn;
    let vos = load_group_vos(db, &payload.group_ids).await?;
    // 按组返回，前端期望 `Record<string, MarkerLinkageVo[]>`
    let mut map: HashMap<String, Vec<MarkerLinkVO>> = HashMap::new();
    for vo in vos {
        let group_id = vo.group_id.clone().unwrap_or_default();
        map.entry(group_id).or_default().push(vo);
    }
    Ok(CommonResponse::new(Ok(serde_json::to_value(map)?)))
}

/// 一个聚合缓存（Java `AccumulatorCache`）：一组关联引用 + 关联 ID → 路径边。
struct AccCache {
    cache_id: String,
    /// (from_id, to_id, linkage_id) —— Java `LinkRefDto`
    links: Vec<(i64, i64, i64)>,
    /// (linkage_id, edges) —— Java `pathMap`
    paths: Vec<(i64, Vec<MarkerLinkagePathEdge>)>,
}

impl AccCache {
    fn new(cache_id: String) -> Self {
        Self {
            cache_id,
            links: Vec::new(),
            paths: Vec::new(),
        }
    }

    /// Java `AccumulatorCache.inLinkage`。
    fn in_linkage(&self, src_from: i64, src_to: i64, use_from: bool, use_to: bool) -> bool {
        let valid_from = src_from > 0;
        let valid_to = src_to > 0;
        if !valid_from && !valid_to {
            return true;
        }
        for &(tar_from, tar_to, _) in &self.links {
            if use_from && valid_from && (tar_from == src_from || tar_from == src_to) {
                return true;
            }
            if use_to && valid_to && (tar_to == src_from || tar_to == src_to) {
                return true;
            }
        }
        false
    }
}

/// Java `MarkerLinkageDataHelper.buildLinkageGraph` 的移植：
/// 按 (groupId, linkAction) 聚合 →（TRIGGER 族合并策略）→（TRIGGER_ALL/ANY
/// 按起点集合再合并）→ 按端点分发 → 组装 `GraphVo{relations,relRefs,pathRefs}`。
/// BTreeMap 保证键序确定（blob MD5 稳定）。
pub(crate) fn build_linkage_graph(vos: &[MarkerLinkVO]) -> BTreeMap<String, serde_json::Value> {
    // 1. 有效过滤 + 按 (group, action) 分组（Java getGraphSearchMap）
    let mut search: BTreeMap<(String, String), Vec<&MarkerLinkVO>> = BTreeMap::new();
    for vo in vos {
        let Some(group_id) = vo.group_id.as_ref().filter(|g| !g.is_empty()) else {
            continue;
        };
        if vo.from_id <= 0 || vo.to_id <= 0 {
            continue;
        }
        let Some(action) = vo.link_action else {
            continue;
        };
        search
            .entry((group_id.clone(), action_str(action)))
            .or_default()
            .push(vo);
    }

    // 2-3. 聚合 + 行为分组（Java accumulateGraphData + groupGraphData）
    let mut grouped: Vec<(String, String, Vec<AccCache>)> = Vec::new();
    for ((group_id, action), list) in &search {
        let mut caches: Vec<AccCache> = Vec::new();
        let mut seq: u64 = 0;
        for vo in list {
            let link_ref = (vo.from_id, vo.to_id, vo.id);
            let cache = match action.as_str() {
                // TRIGGER：一对一，每条关联一个独立缓存
                "TRIGGER" => {
                    seq += 1;
                    caches.push(AccCache::new(format!("c{seq}")));
                    caches.last_mut().expect("just pushed")
                },
                // TRIGGER_ALL / TRIGGER_ANY：任一缓存命中 toId 即并入
                "TRIGGER_ALL" | "TRIGGER_ANY" => {
                    if let Some(pos) = caches
                        .iter()
                        .position(|c| c.in_linkage(0, vo.to_id, false, true))
                    {
                        &mut caches[pos]
                    } else {
                        seq += 1;
                        caches.push(AccCache::new(format!("c{seq}")));
                        caches.last_mut().expect("just pushed")
                    }
                },
                // 分组族（RELATED/DIRECTED/PATH_*/EQUIVALENT）：同对并入同缓存
                _ => {
                    if let Some(pos) = caches
                        .iter()
                        .position(|c| c.in_linkage(vo.from_id, vo.to_id, true, true))
                    {
                        &mut caches[pos]
                    } else {
                        seq += 1;
                        caches.push(AccCache::new(format!("c{seq}")));
                        caches.last_mut().expect("just pushed")
                    }
                },
            };
            if !cache.links.contains(&link_ref) {
                cache.links.push(link_ref);
            }
            let edges = vo
                .path
                .as_ref()
                .map(|p| p.iter().flatten().cloned().collect())
                .unwrap_or_default();
            if !cache.paths.iter().any(|(id, _)| *id == vo.id) {
                cache.paths.push((vo.id, edges));
            }
        }
        // TRIGGER_ALL/ANY：共享起点集合的缓存再合并（Java groupByTriggers）
        let caches = if action == "TRIGGER_ALL" || action == "TRIGGER_ANY" {
            group_by_triggers(caches)
        } else {
            caches
        };
        grouped.push((group_id.clone(), action.clone(), caches));
    }

    // 4-5. 分发 + 组装（Java distributeGraphData + restructureGraphData）
    let mut graphs: BTreeMap<String, GraphBuilder> = BTreeMap::new();
    for (group_id, action, caches) in grouped {
        for cache in caches {
            let relation_id = cache.cache_id.clone();
            let mut relation = serde_json::Map::new();
            relation.insert("type".into(), serde_json::json!(action));
            let is_trigger_family =
                matches!(action.as_str(), "TRIGGER" | "TRIGGER_ALL" | "TRIGGER_ANY");
            if is_trigger_family {
                relation.insert(
                    "triggers".into(),
                    dedup_json(&cache.links, |(from, _, link_id)| {
                        serde_json::json!({ "markerId": from, "pathRefId": link_id })
                    }),
                );
                relation.insert(
                    "targets".into(),
                    dedup_json(&cache.links, |(_, to, link_id)| {
                        serde_json::json!({ "markerId": to, "pathRefId": link_id })
                    }),
                );
            } else {
                relation.insert(
                    "group".into(),
                    dedup_json(&cache.links, |(from, to, link_id)| {
                        serde_json::json!({ "srcId": from, "tarId": to, "pathRefId": link_id })
                    }),
                );
            }

            let builder = graphs.entry(group_id.clone()).or_default();
            // 同一 markerId 的重复端点去重后挂 relationId（Java distKey 语义）
            let mut endpoints: Vec<i64> = cache
                .links
                .iter()
                .flat_map(|(from, to, _)| [*from, *to])
                .collect();
            endpoints.sort_unstable();
            endpoints.dedup();
            for mid in endpoints {
                builder
                    .relations
                    .entry(mid)
                    .or_default()
                    .push(relation_id.clone());
            }
            builder
                .rel_refs
                .insert(relation_id, serde_json::Value::Object(relation));
            for (link_id, edges) in cache.paths {
                builder.path_refs.insert(link_id, edges);
            }
        }
    }

    graphs
        .into_iter()
        .map(|(group_id, builder)| {
            let vo = serde_json::json!({
                "relations": builder
                    .relations
                    .iter()
                    .map(|(mid, rels)| (mid.to_string(), serde_json::json!(rels)))
                    .collect::<serde_json::Map<String, serde_json::Value>>(),
                "relRefs": builder.rel_refs,
                "pathRefs": builder
                    .path_refs
                    .iter()
                    .map(|(lid, edges)| (lid.to_string(), serde_json::json!(edges)))
                    .collect::<serde_json::Map<String, serde_json::Value>>(),
            });
            (group_id, vo)
        })
        .collect()
}

/// Java `MarkerLinkageGraphGrouper.groupByTriggers`：起点集合（排序哈希）相同
/// 的缓存合并为一个。
fn group_by_triggers(caches: Vec<AccCache>) -> Vec<AccCache> {
    let mut key_map: HashMap<String, String> = HashMap::new();
    let mut cache_map: HashMap<String, AccCache> = HashMap::new();
    for cache in caches {
        let mut from_ids: Vec<i64> = cache
            .links
            .iter()
            .map(|(from, _, _)| *from)
            .filter(|id| *id > 0)
            .collect();
        from_ids.sort_unstable();
        from_ids.dedup();
        if from_ids.is_empty() {
            continue;
        }
        let hash = from_ids
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let canonical = key_map
            .get(&hash)
            .cloned()
            .unwrap_or_else(|| cache.cache_id.clone());
        let entry = cache_map
            .entry(canonical.clone())
            .or_insert_with(|| AccCache::new(canonical.clone()));
        for link in cache.links {
            if !entry.links.contains(&link) {
                entry.links.push(link);
            }
        }
        for path in cache.paths {
            if !entry.paths.iter().any(|(id, _)| *id == path.0) {
                entry.paths.push(path);
            }
        }
        key_map.insert(hash, canonical);
    }
    cache_map.into_values().collect()
}

/// 组装中的 GraphVo（Java `GraphDto`）。
#[derive(Default)]
struct GraphBuilder {
    relations: BTreeMap<i64, Vec<String>>,
    rel_refs: serde_json::Map<String, serde_json::Value>,
    path_refs: BTreeMap<i64, Vec<MarkerLinkagePathEdge>>,
}

/// 关联引用集 → 去重后的 JSON 数组（Java `Set<LinkRefVo>` 语义）。
fn dedup_json(
    links: &[(i64, i64, i64)],
    project: impl Fn(&(i64, i64, i64)) -> serde_json::Value,
) -> serde_json::Value {
    let mut seen: Vec<serde_json::Value> = Vec::new();
    for link in links {
        let v = project(link);
        if !seen.contains(&v) {
            seen.push(v);
        }
    }
    serde_json::Value::Array(seen)
}

/// 图数据中的路径边坐标回填（Java `patchPathMarkerCoordsInGraph`）。
fn patch_graph_coords(
    graphs: &mut BTreeMap<String, serde_json::Value>,
    coords: &HashMap<i64, (f64, f64)>,
) {
    for graph in graphs.values_mut() {
        let Some(path_refs) = graph.get_mut("pathRefs").and_then(|v| v.as_object_mut()) else {
            continue;
        };
        for edges in path_refs.values_mut() {
            let Some(arr) = edges.as_array_mut() else {
                continue;
            };
            for edge in arr.iter_mut() {
                patch_edge_coords(edge, coords);
            }
        }
    }
}

fn patch_edge_coords(edge: &mut serde_json::Value, coords: &HashMap<i64, (f64, f64)>) {
    let Some(obj) = edge.as_object_mut() else {
        return;
    };
    let id1 = obj.get("id1").and_then(|v| v.as_i64()).unwrap_or(0);
    let id2 = obj.get("id2").and_then(|v| v.as_i64()).unwrap_or(0);
    if let Some(&(x, y)) = coords.get(&id1) {
        obj.insert("x1".into(), serde_json::json!(x));
        obj.insert("y1".into(), serde_json::json!(y));
    }
    if let Some(&(x, y)) = coords.get(&id2) {
        obj.insert("x2".into(), serde_json::json!(x));
        obj.insert("y2".into(), serde_json::json!(y));
    }
}

pub async fn do_get_graph(
    _auth: AuthInfo,
    payload: MarkerLinkGraphRequest,
) -> Result<CommonResponse<serde_json::Value>> {
    // 与 Java 实现一致：未指定组 ID 时返回空 map
    if payload.group_ids.is_empty() {
        return Ok(CommonResponse::new(Ok(serde_json::json!({}))));
    }
    let db = &DB_CONN.wait().pg_conn;
    let vos = load_group_vos(db, &payload.group_ids).await?;
    let mut graphs = build_linkage_graph(&vos);
    // 坐标回填（Java graphMarkerLinkage 末段）
    let mut path_ids: Vec<i64> = Vec::new();
    for vo in &vos {
        for edge in vo.path.iter().flatten().flatten() {
            for id in [edge.id1.unwrap_or(0), edge.id2.unwrap_or(0)] {
                if id > 0 && !path_ids.contains(&id) {
                    path_ids.push(id);
                }
            }
        }
    }
    let coords = path_marker_coords(db, &path_ids).await?;
    patch_graph_coords(&mut graphs, &coords);
    Ok(CommonResponse::new(Ok(serde_json::to_value(graphs)?)))
}

pub async fn do_delete(
    auth: AuthInfo,
    payload: MarkerLinkDeleteRequest,
) -> Result<CommonResponse<serde_json::Value>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    // 收集被删除关联涉及的组 ID 与点位 ID，返回给前端刷新本地数据
    let mut groups: Vec<String> = Vec::new();
    let mut markers: Vec<i64> = Vec::new();
    let mut collect_affected = |it: &linkage_model::Model| {
        if !it.group_id.is_empty() && !groups.contains(&it.group_id) {
            groups.push(it.group_id.clone());
        }
        if it.from_id > 0 && !markers.contains(&it.from_id) {
            markers.push(it.from_id);
        }
        if it.to_id > 0 && !markers.contains(&it.to_id) {
            markers.push(it.to_id);
        }
    };
    if let Some(ids) = payload.ids {
        for id in ids {
            if let Some(item) = linkage_model::Entity::find_safety_by_id(id).one(db).await? {
                collect_affected(&item);
                let mut am: linkage_model::ActiveModel = item.into();
                am.del_flag = Set(true);
                // 审计字段：软删也是修改，设置 update 组
                am.updater_id = Set(Some(auth.info.id));
                linkage_model::Entity::delete_safety(am)?.exec(db).await?;
            }
        }
    }

    if let Some(group_ids) = payload.group_ids {
        for gid in group_ids {
            let items = linkage_model::Entity::find_safety()
                .filter(linkage_model::Column::GroupId.eq(gid))
                .all(db)
                .await?;
            for it in items {
                collect_affected(&it);
                let mut am: linkage_model::ActiveModel = it.into();
                am.del_flag = Set(true);
                // 审计字段：软删也是修改，设置 update 组
                am.updater_id = Set(Some(auth.info.id));
                linkage_model::Entity::delete_safety(am)?.exec(db).await?;
            }
        }
    }
    super::binary_doc::invalidate_doc_cache().await;
    super::super::ws::ws_broadcast(
        "MarkerLinkageDeleted",
        serde_json::json!({
            "groups": groups,
            "markers": markers
        }),
    );
    super::super::ws::ws_broadcast_debounced(
        "MarkerBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    super::super::ws::ws_broadcast_debounced(
        "MarkerLinkageBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(CommonResponse::new(Ok(serde_json::json!({
        "groups": groups,
        "markers": markers
    }))))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vo(
        group: &str,
        id: i64,
        from: i64,
        to: i64,
        action: MarkerLinkageLinkAction,
    ) -> MarkerLinkVO {
        MarkerLinkVO {
            version: 0,
            id,
            group_id: Some(group.into()),
            from_id: from,
            to_id: to,
            link_action: Some(action),
            link_reverse: Some(false),
            path: None,
            creator_id: None,
            updater_id: None,
            update_time: None,
        }
    }

    #[test]
    fn reverse_swaps_and_resets_flag() {
        let mut v = vo("g", 1, 5, 9, MarkerLinkageLinkAction::Trigger);
        v.link_reverse = Some(true);
        reverse_linkage_vos(std::slice::from_mut(&mut v));
        assert_eq!((v.from_id, v.to_id), (9, 5));
        assert_eq!(v.link_reverse, Some(false));
    }

    #[test]
    fn pair_key_is_order_insensitive() {
        assert_eq!(pair_key(9, 5), pair_key(5, 9));
        assert_eq!(pair_key(9, 5), (5, 9));
    }

    #[test]
    fn graph_trigger_is_one_relation_per_link() {
        let vos = vec![
            vo("g", 1, 100, 200, MarkerLinkageLinkAction::Trigger),
            vo("g", 2, 100, 300, MarkerLinkageLinkAction::Trigger),
        ];
        let graphs = build_linkage_graph(&vos);
        let g = &graphs["g"];
        let relations = g.get("relations").unwrap();
        // 每个 markerId 恰好挂两条 relation（一对一语义）
        assert_eq!(relations.get("100").unwrap().as_array().unwrap().len(), 2);
        assert_eq!(relations.get("200").unwrap().as_array().unwrap().len(), 1);
        // relRefs：trigger 族含 triggers/targets，无 group
        let rel_refs = g.get("relRefs").unwrap().as_object().unwrap();
        assert_eq!(rel_refs.len(), 2);
        for rel in rel_refs.values() {
            assert!(rel.get("triggers").is_some());
            assert!(rel.get("targets").is_some());
            assert!(rel.get("group").is_none());
        }
    }

    #[test]
    fn graph_group_family_shares_one_relation_per_pair_group() {
        let vos = vec![
            vo("g", 1, 100, 200, MarkerLinkageLinkAction::Related),
            vo("g", 2, 200, 300, MarkerLinkageLinkAction::Related),
        ];
        let graphs = build_linkage_graph(&vos);
        let g = &graphs["g"];
        // 两条 Related 关联共享 fromId=200 的匹配 → 并入同一缓存 → 单 relation
        let rel_refs = g.get("relRefs").unwrap().as_object().unwrap();
        assert_eq!(rel_refs.len(), 1);
        let rel = rel_refs.values().next().unwrap();
        assert_eq!(rel.get("type").unwrap(), "RELATED");
        let group = rel.get("group").unwrap().as_array().unwrap();
        assert_eq!(group.len(), 2);
    }

    #[test]
    fn graph_trigger_all_merges_by_shared_target_then_from_set() {
        let vos = vec![
            vo("g", 1, 100, 300, MarkerLinkageLinkAction::TriggerAll),
            vo("g", 2, 100, 301, MarkerLinkageLinkAction::TriggerAll),
            vo("g", 3, 200, 302, MarkerLinkageLinkAction::TriggerAll),
        ];
        let graphs = build_linkage_graph(&vos);
        let g = &graphs["g"];
        // 前两条 toId 不同、起点集合相同（{100}）→ grouper 合并为一个 relation；
        // 第三条起点集合不同 → 另一个 relation。
        let rel_refs = g.get("relRefs").unwrap().as_object().unwrap();
        assert_eq!(rel_refs.len(), 2);
        let merged = rel_refs
            .values()
            .map(|r| r.get("triggers").unwrap().as_array().unwrap().len())
            .collect::<Vec<_>>();
        assert!(
            merged.contains(&2),
            "merged cache holds 2 triggers: {merged:?}"
        );
    }
}
