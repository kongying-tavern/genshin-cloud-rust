use anyhow::{Result, anyhow};
use chrono::Utc;

use sea_orm::{
    ActiveValue::{NotSet, Set},
    QuerySelect,
    prelude::*,
};

use std::collections::HashSet;

use _database::{
    DB_CONN, models::item::item as item_model, models::item::item_type_link as itl_model,
    models::marker::marker as marker_model, models::marker::marker_item_link as mil_model,
    models::marker::marker_linkage as linkage_model,
};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        marker::MarkerFilterRequest,
        marker::{
            MarkerAddRequest, MarkerItemLinkVo, MarkerTweakConfigPropEnum, MarkerTweakRequest,
            MarkerUpdateData,
        },
        marker::{MarkerEmptyResponse, MarkerListResponse, MarkerVO},
        wrapper::{CommonResponse, Pagination},
    },
};

/// 批量读取点位物品关联（`marker_item_link` + item 的 `icon_tag`），
/// 返回 marker_id → itemList。避免逐点查询的 N+1。
pub(crate) async fn marker_item_map(
    db: &sea_orm::DatabaseConnection,
    marker_ids: &[i64],
) -> Result<std::collections::HashMap<i64, Vec<MarkerItemLinkVo>>> {
    let mut map: std::collections::HashMap<i64, Vec<MarkerItemLinkVo>> =
        std::collections::HashMap::new();
    if marker_ids.is_empty() {
        return Ok(map);
    }
    let mut links: Vec<mil_model::Model> = Vec::new();
    for chunk in marker_ids.chunks(1000) {
        links.extend(
            mil_model::Entity::find_safety()
                .filter(mil_model::Column::MarkerId.is_in(chunk))
                .all(db)
                .await?,
        );
    }
    let item_ids: Vec<i64> = links.iter().map(|l| l.item_id).collect();
    let mut item_icon_ids: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
    if !item_ids.is_empty() {
        for chunk in item_ids.chunks(1000) {
            for it in item_model::Entity::find_safety()
                .filter(item_model::Column::Id.is_in(chunk))
                .all(db)
                .await?
            {
                item_icon_ids.insert(it.id, it.icon_id);
            }
        }
    }
    let icon_tag_map = super::icon::icon_tag_map(db).await?;
    for l in links {
        let icon_id = item_icon_ids.get(&l.item_id).copied().unwrap_or(0);
        map.entry(l.marker_id).or_default().push(MarkerItemLinkVo {
            item_id: l.item_id,
            count: l.count,
            icon_id,
            icon_tag: Some(icon_tag_map.get(&icon_id).cloned().unwrap_or_default()),
        });
    }
    Ok(map)
}

/// 批量读取点位归属的连线组（`marker_linkage` 的 from_id/to_id），
/// 返回 marker_id → group_id（同一 marker 命中多组时取第一条）。避免逐点查询的 N+1。
pub(crate) async fn marker_linkage_map(
    db: &sea_orm::DatabaseConnection,
    marker_ids: &[i64],
) -> Result<std::collections::HashMap<i64, String>> {
    let mut map: std::collections::HashMap<i64, String> = std::collections::HashMap::new();
    if marker_ids.is_empty() {
        return Ok(map);
    }
    let mut links: Vec<linkage_model::Model> = Vec::new();
    for chunk in marker_ids.chunks(1000) {
        links.extend(
            linkage_model::Entity::find_safety()
                .filter(
                    sea_orm::Condition::any()
                        .add(linkage_model::Column::FromId.is_in(chunk))
                        .add(linkage_model::Column::ToId.is_in(chunk)),
                )
                .all(db)
                .await?,
        );
    }
    for l in links {
        let gid = l.group_id;
        map.entry(l.from_id).or_insert_with(|| gid.clone());
        map.entry(l.to_id).or_insert_with(|| gid.clone());
    }
    Ok(map)
}

/// 批量调整点位数据，目前实现常用字段的替换/更新逻辑：
/// 对于复杂的 item_list 调整暂时跳过（可在后续增强）。
fn model_to_vo(
    it: marker_model::Model,
    item_map: &std::collections::HashMap<i64, Vec<MarkerItemLinkVo>>,
    linkage_map: Option<&std::collections::HashMap<i64, String>>,
) -> MarkerVO {
    MarkerVO {
        version: it.version,
        id: it.id,
        create_time: it.create_time.and_utc().timestamp_millis() as f64,
        update_time: it
            .update_time
            .map(|dt| dt.and_utc().timestamp_millis() as f64),
        creator_id: it.creator_id,
        updater_id: it.updater_id,
        del_flag: it.del_flag,
        marker_stamp: it.marker_stamp,
        marker_title: it.marker_title,
        position: it.position,
        content: it.content,
        picture: it.picture,
        marker_creator_id: it.marker_creator_id,
        picture_creator_id: it.picture_creator_id,
        video_path: it.video_path,
        refresh_time: it.refresh_time,
        hidden_flag: it.hidden_flag,
        extra: it.extra,
        item_list: item_map.get(&it.id).cloned().unwrap_or_default(),
        // 列表接口传入 marker_id → group_id map 回填；tweak 等场景可传 None
        linkage_id: linkage_map.and_then(|m| m.get(&it.id).cloned()),
    }
}

/// camelCase marker view for the BinaryMD5 pages (the wire contract of the
/// `marker_doc` blob is the Java `MarkerVo` naming).
pub(crate) fn model_to_vo_doc(
    it: &marker_model::Model,
    item_map: &std::collections::HashMap<i64, Vec<MarkerItemLinkVo>>,
    linkage_map: Option<&std::collections::HashMap<i64, String>>,
) -> MarkerVO {
    model_to_vo(it.clone(), item_map, linkage_map)
}

/// 文本类 tweak 的"新值"：meta.replace 优先，其次 meta.value 中的 String。
fn tweak_text_value(tweak: &_utils::models::marker::MarkerTweakConfig) -> Option<String> {
    if let Some(v) = &tweak.meta.replace {
        return Some(v.clone());
    }
    if let Some(_utils::models::marker::TweakValue::String(s)) = &tweak.meta.value {
        return Some(s.clone());
    }
    None
}

/// 文本类 tweak 的"检查文本"（待匹配/待处理子串）：meta.test 优先，其次 meta.value 中的 String。
/// 前端 RemoveLeft/RemoveRight/TrimLeft/TrimRight 只发 test（待处理子串），
/// Replace/Update/ReplaceRegex 的条件替换同样使用 test。
fn tweak_text_needle(tweak: &_utils::models::marker::MarkerTweakConfig) -> Option<String> {
    if let Some(v) = &tweak.meta.test {
        return Some(v.clone());
    }
    if let Some(_utils::models::marker::TweakValue::String(s)) = &tweak.meta.value {
        return Some(s.clone());
    }
    None
}

/// 数值类 tweak 的整数值：Integer 直接取；Double 且为整数值（如 1.0）时取整。
/// 前端可能发送 1.0 这类浮点字面量，仅匹配 Integer 会静默失效。
fn tweak_int_value(value: &_utils::models::marker::TweakValue) -> Option<i64> {
    match value {
        _utils::models::marker::TweakValue::Integer(i) => Some(*i),
        _utils::models::marker::TweakValue::Double(d) if d.fract() == 0.0 => Some(*d as i64),
        _ => None,
    }
}

/// 按 tweak 类型计算文本字段（title/content）的新值：
/// - Update：条件编辑 —— meta.test 命中（origin 包含 test）时，将 test 的全部出现处
///   替换为 meta.replace（或 value），未命中返回 None（不修改）；test 缺失时按 Replace 整值替换
/// - Replace：meta.test 存在时 replaceAll（origin 中所有 test 出现处 → 新值）；
///   test 缺失时整值替换（默认，前端标题编辑）
/// - ReplaceRegex：regex 仅为 Cargo.lock 传递依赖（无 crate 声明），以字面量替换近似
///   （test → 新值，全部出现），与前端真实正则预览不一致已知
/// - Prepend / Append：新值 + 原值 / 原值 + 新值
/// - RemoveLeft / RemoveRight：从开头/结尾移除 test（或 value）子串一次，未命中保持原值
/// - TrimLeft / TrimRight：无 test/value 时去除首/尾空白字符（对应 Java StrUtil.trimStart/End）；
///   有 test（或 value）时剥离开头/结尾重复出现的该子串
/// - 其余类型（InsertIfAbsent/InsertOrUpdate/Merge 等）对文本字段无意义，返回 None（不修改）
fn apply_text_tweak(
    tweak: &_utils::models::marker::MarkerTweakConfig,
    origin: Option<String>,
) -> Option<String> {
    let origin = origin.unwrap_or_default();
    match tweak.marker_tweak_config_type {
        _utils::models::marker::MarkerTweakConfigTypeEnum::Replace => {
            if let Some(needle) = tweak_text_needle(tweak)
                && !needle.is_empty()
                && let Some(rep) = tweak_text_value(tweak)
            {
                Some(origin.replace(needle.as_str(), rep.as_str()))
            } else {
                tweak_text_value(tweak)
            }
        },
        _utils::models::marker::MarkerTweakConfigTypeEnum::Update => {
            if let Some(needle) = tweak_text_needle(tweak)
                && !needle.is_empty()
            {
                if origin.contains(needle.as_str()) {
                    let rep = tweak_text_value(tweak)?;
                    Some(origin.replace(needle.as_str(), rep.as_str()))
                } else {
                    None
                }
            } else {
                tweak_text_value(tweak)
            }
        },
        // regex 仅存在于 Cargo.lock 传递依赖中（functions 未声明），无法使用真实正则，
        // 以字面量字符串替换近似（与 Replace 带 test 时语义一致）；前端正则预览不一致已知
        _utils::models::marker::MarkerTweakConfigTypeEnum::ReplaceRegex => {
            if let Some(needle) = tweak_text_needle(tweak)
                && !needle.is_empty()
                && let Some(rep) = tweak_text_value(tweak)
            {
                Some(origin.replace(needle.as_str(), rep.as_str()))
            } else {
                None
            }
        },
        _utils::models::marker::MarkerTweakConfigTypeEnum::Prepend => {
            tweak_text_value(tweak).map(|v| v + &origin)
        },
        _utils::models::marker::MarkerTweakConfigTypeEnum::Append => {
            tweak_text_value(tweak).map(|v| origin + &v)
        },
        _utils::models::marker::MarkerTweakConfigTypeEnum::RemoveLeft => {
            let needle = tweak_text_needle(tweak)?;
            Some(
                origin
                    .strip_prefix(needle.as_str())
                    .unwrap_or(origin.as_str())
                    .to_owned(),
            )
        },
        _utils::models::marker::MarkerTweakConfigTypeEnum::RemoveRight => {
            let needle = tweak_text_needle(tweak)?;
            Some(
                origin
                    .strip_suffix(needle.as_str())
                    .unwrap_or(origin.as_str())
                    .to_owned(),
            )
        },
        _utils::models::marker::MarkerTweakConfigTypeEnum::TrimLeft => {
            // 前端发空 meta 时按空白字符剥离（对应 Java StrUtil.trimStart）
            match tweak_text_needle(tweak) {
                Some(n) if !n.is_empty() => Some(origin.trim_start_matches(n.as_str()).to_owned()),
                _ => Some(origin.trim_start().to_owned()),
            }
        },
        _utils::models::marker::MarkerTweakConfigTypeEnum::TrimRight => {
            // 前端发空 meta 时按空白字符剥离（对应 Java StrUtil.trimEnd）
            match tweak_text_needle(tweak) {
                Some(n) if !n.is_empty() => Some(origin.trim_end_matches(n.as_str()).to_owned()),
                _ => Some(origin.trim_end().to_owned()),
            }
        },
        // InsertIfAbsent / InsertOrUpdate / Merge 等其余类型：对文本字段无意义，忽略
        _ => None,
    }
}

pub async fn do_tweak(
    auth: AuthInfo,
    payloads: Vec<MarkerTweakRequest>,
) -> Result<CommonResponse<Vec<MarkerVO>>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;

    let mut touched_ids: Vec<i64> = Vec::new();
    for payload in payloads {
        for marker_id in payload.marker_ids.iter() {
            let m = marker_model::Entity::find_safety_by_id(*marker_id)
                .one(db)
                .await?;
            if m.is_none() {
                // 跳过缺失的标记
                continue;
            }
            let m = m.unwrap();
            let content = m.content.clone();
            let marker_title = m.marker_title.clone();
            let mut am: marker_model::ActiveModel = m.into();

            // 同一字段的多条 tweak 按顺序链式应用（前一条结果作为下一条的 origin）：
            // 先按字段分组，组内依次 apply（结果传递），避免每条都基于原值导致中间结果丢失。
            let mut groups: Vec<(
                MarkerTweakConfigPropEnum,
                Vec<&_utils::models::marker::MarkerTweakConfig>,
            )> = Vec::new();
            for tweak in payload.tweaks.iter() {
                match groups.iter_mut().find(|(prop, _)| *prop == tweak.prop) {
                    Some((_, list)) => list.push(tweak),
                    None => groups.push((tweak.prop.clone(), vec![tweak])),
                }
            }

            for (prop, tweaks) in groups {
                match prop {
                    MarkerTweakConfigPropEnum::Content => {
                        let mut cur = content.clone();
                        for tweak in tweaks {
                            if let Some(next) = apply_text_tweak(tweak, cur.clone()) {
                                cur = Some(next.clone());
                                am.content = Set(Some(next));
                            }
                        }
                    },
                    MarkerTweakConfigPropEnum::Title => {
                        let mut cur = marker_title.clone();
                        for tweak in tweaks {
                            if let Some(next) = apply_text_tweak(tweak, cur.clone()) {
                                cur = Some(next.clone());
                                am.marker_title = Set(Some(next));
                            }
                        }
                    },
                    MarkerTweakConfigPropEnum::Position => {
                        for tweak in tweaks {
                            if let Some(v) = &tweak.meta.replace {
                                am.position = Set(v.clone());
                            } else if let Some(_utils::models::marker::TweakValue::String(s)) =
                                &tweak.meta.value
                            {
                                // 前端拖拽移动发 meta.value（"x,y" 字符串，与 replace 同语义）
                                am.position = Set(s.clone());
                            }
                        }
                    },
                    MarkerTweakConfigPropEnum::VideoPath => {
                        for tweak in tweaks {
                            if let Some(v) = &tweak.meta.replace {
                                am.video_path = Set(Some(v.clone()));
                            }
                        }
                    },
                    MarkerTweakConfigPropEnum::RefreshTime => {
                        for tweak in tweaks {
                            if let Some(v) = &tweak.meta.value
                                && let Some(i) = tweak_int_value(v)
                            {
                                am.refresh_time = Set(i);
                            }
                        }
                    },
                    MarkerTweakConfigPropEnum::Extra => {
                        for tweak in tweaks {
                            if let Some(map) = &tweak.meta.map {
                                // 用序列化后的 map 完整替换 extra
                                am.extra = Set(Some(serde_json::to_value(map)?));
                            } else if let Some(_utils::models::marker::TweakValue::AnythingMap(m)) =
                                &tweak.meta.value
                            {
                                // 尝试设置任意 JSON 值
                                am.extra = Set(Some(serde_json::to_value(m)?));
                            }
                        }
                    },
                    MarkerTweakConfigPropEnum::HiddenFlag => {
                        for tweak in tweaks {
                            if let Some(val) = &tweak.meta.value
                                && let Some(i) = tweak_int_value(val)
                            {
                                // HiddenFlag 是一个枚举；utils 中定义。尝试从整数转换。
                                let hf = match i as i32 {
                                    0 => _utils::types::HiddenFlag::Visible,
                                    1 => _utils::types::HiddenFlag::Hidden,
                                    2 => _utils::types::HiddenFlag::Spy,
                                    3 => _utils::types::HiddenFlag::Suprise,
                                    _ => _utils::types::HiddenFlag::Visible,
                                };
                                am.hidden_flag = Set(hf);
                            }
                        }
                    },
                    MarkerTweakConfigPropEnum::ItemList => {
                        for tweak in tweaks {
                            tweak_item_list(db, *marker_id, tweak).await?;
                        }
                    },
                }
            }

            // 通过 ActiveModelBehavior 设置 updater 与 update_time；确保携带版本信息
            marker_model::Entity::update_safety(am)?.exec(db).await?;
            touched_ids.push(*marker_id);
        }
    }

    // 返回被修改 marker 的 VO 列表
    if touched_ids.is_empty() {
        return Ok(CommonResponse::new(Ok(vec![])));
    }
    super::binary_doc::invalidate_doc_cache().await;
    let item_map = marker_item_map(db, &touched_ids).await?;
    let mut arr = Vec::new();
    for chunk in touched_ids.chunks(1000) {
        let items = marker_model::Entity::find_safety()
            .filter(marker_model::Column::Id.is_in(chunk))
            .all(db)
            .await?;
        for it in items {
            arr.push(model_to_vo(it, &item_map, None));
        }
    }
    Ok(CommonResponse::new(Ok(arr)))
}

/// 解析 tweak 的 item_list 元数据为 (item_id, count) 列表。
/// 支持裸数字 id 或 `{"id": n, "count": c}` 对象。
fn parse_item_entries(item_list: &[Option<serde_json::Value>]) -> Vec<(i64, i32)> {
    let mut ret = Vec::new();
    for v in item_list.iter().flatten() {
        match v {
            serde_json::Value::Number(n) => {
                if let Some(id) = n.as_i64() {
                    ret.push((id, 1));
                }
            },
            serde_json::Value::Object(obj) => {
                // 兼容 `id`（tweak 裸 id 对象）与 `itemId`（前端 MarkerItemLinkVo）
                if let Some(id) = obj
                    .get("id")
                    .or_else(|| obj.get("itemId"))
                    .and_then(|x| x.as_i64())
                {
                    let count = obj.get("count").and_then(|x| x.as_i64()).unwrap_or(1) as i32;
                    ret.push((id, count));
                }
            },
            _ => {},
        }
    }
    ret
}

/// 批量调整某 marker 的 item 关联（marker_item_link 表）。
/// 支持的调整类型：Append / Prepend（仅插入缺失条目，去重）、
/// InsertIfAbsent / InsertOrUpdate / Merge / Update（追加或更新 count）、
/// Replace（整表替换）、RemoveLeft / RemoveRight（移除列出的关联）、
/// TrimLeft / TrimRight（只保留列出的关联）。
async fn tweak_item_list(
    db: &sea_orm::DatabaseConnection,
    marker_id: i64,
    tweak: &_utils::models::marker::MarkerTweakConfig,
) -> Result<()> {
    use _utils::models::marker::{MarkerTweakConfigTypeEnum, TweakMeta};

    let TweakMeta {
        item_list: Some(item_list),
        ..
    } = &tweak.meta
    else {
        return Ok(());
    };
    let entries = parse_item_entries(item_list);
    if entries.is_empty() {
        return Ok(());
    }

    let existing = mil_model::Entity::find_safety()
        .filter(mil_model::Column::MarkerId.eq(marker_id))
        .all(db)
        .await?;
    let mut existing_map: std::collections::HashMap<i64, mil_model::Model> =
        existing.into_iter().map(|l| (l.item_id, l)).collect();

    match tweak.marker_tweak_config_type {
        MarkerTweakConfigTypeEnum::Replace => {
            // 软删现有全部关联，然后插入新列表
            for (_item_id, link) in existing_map.drain() {
                mil_model::Entity::delete_safety(link.into())?
                    .exec(db)
                    .await?;
            }
            for (item_id, count) in entries {
                insert_item_link(db, marker_id, item_id, count).await?;
            }
        },
        MarkerTweakConfigTypeEnum::RemoveLeft | MarkerTweakConfigTypeEnum::RemoveRight => {
            for (item_id, _count) in entries {
                if let Some(link) = existing_map.remove(&item_id) {
                    mil_model::Entity::delete_safety(link.into())?
                        .exec(db)
                        .await?;
                }
            }
        },
        // Append / Prepend：仅插入缺失条目（去重），已存在的不动。
        // marker_item_link 无排序列，Prepend 与 Append 落库结果等价（顺序由查询方决定）。
        MarkerTweakConfigTypeEnum::Append | MarkerTweakConfigTypeEnum::Prepend => {
            for (item_id, count) in entries {
                if !existing_map.contains_key(&item_id) {
                    insert_item_link(db, marker_id, item_id, count).await?;
                }
            }
        },
        // Trim：只保留 item_list 中列出的关联（其余软删）。
        MarkerTweakConfigTypeEnum::TrimLeft | MarkerTweakConfigTypeEnum::TrimRight => {
            let keep: HashSet<i64> = entries.iter().map(|(id, _)| *id).collect();
            for (item_id, link) in existing_map.iter() {
                if !keep.contains(item_id) {
                    mil_model::Entity::delete_safety(link.clone().into())?
                        .exec(db)
                        .await?;
                }
            }
        },
        // InsertIfAbsent / InsertOrUpdate / Merge / Update：
        // 追加或更新 count；InsertIfAbsent 对已存在条目跳过。
        // 其余类型（ReplaceRegex 等）对 item 列表无意义，忽略。
        MarkerTweakConfigTypeEnum::InsertIfAbsent
        | MarkerTweakConfigTypeEnum::InsertOrUpdate
        | MarkerTweakConfigTypeEnum::Merge
        | MarkerTweakConfigTypeEnum::Update => {
            for (item_id, count) in entries {
                if let Some(link) = existing_map.get(&item_id) {
                    // 已存在：仅 InsertIfAbsent 跳过；其余更新 count
                    if matches!(
                        tweak.marker_tweak_config_type,
                        MarkerTweakConfigTypeEnum::InsertIfAbsent
                    ) {
                        continue;
                    }
                    let mut am: mil_model::ActiveModel = link.clone().into();
                    am.count = Set(count);
                    mil_model::Entity::update_safety(am)?.exec(db).await?;
                } else {
                    insert_item_link(db, marker_id, item_id, count).await?;
                }
            }
        },
        _ => {},
    }
    Ok(())
}

async fn insert_item_link(
    db: &sea_orm::DatabaseConnection,
    marker_id: i64,
    item_id: i64,
    count: i32,
) -> Result<()> {
    let now = Utc::now().naive_utc();
    let am = mil_model::ActiveModel {
        version: Set(0),
        // id 为 IDENTITY 列：NotSet 走自增，避免多条插入共用 id=0 撞主键
        id: sea_orm::ActiveValue::NotSet,
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),
        item_id: Set(item_id),
        marker_id: Set(marker_id),
        count: Set(count),
    };
    mil_model::Entity::insert(am).exec(db).await?;
    Ok(())
}

pub async fn do_add_single(
    auth: AuthInfo,
    payload: MarkerAddRequest,
) -> Result<CommonResponse<i64>> {
    auth.require_non_anonymous()?;
    let now = Utc::now().naive_utc();
    let db = &DB_CONN.wait().pg_conn;

    let active = marker_model::ActiveModel {
        version: Set(0),
        id: NotSet,
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),

        marker_title: Set(Some(payload.marker_title)),
        position: Set(payload.position),
        content: Set(payload.content),
        picture: Set(payload.picture),
        marker_creator_id: Set(payload.marker_creator_id),
        picture_creator_id: Set(payload.picture_creator_id),
        video_path: Set(payload.video_path),
        refresh_time: Set(payload.refresh_time.unwrap_or(0)),
        hidden_flag: Set(payload.hidden_flag),
        extra: Set(payload
            .extra
            .map(|m| serde_json::to_value(m).unwrap_or(serde_json::json!({})))),
        ..Default::default()
    };

    let res = active.insert(db).await?;
    // item_list 落库（parse_item_entries 支持裸数字 / {id|itemId, count}）
    for (item_id, count) in parse_item_entries(&payload.item_list) {
        insert_item_link(db, res.id, item_id, count).await?;
    }
    super::binary_doc::invalidate_doc_cache().await;
    // 直接返回裸 id，前端期望 data 为 number
    Ok(CommonResponse::new(Ok(res.id)))
}

pub async fn do_update_single(
    auth: AuthInfo,
    payload: MarkerUpdateData,
) -> Result<CommonResponse<MarkerEmptyResponse>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;

    let m = marker_model::Entity::find_safety_by_id(payload.id)
        .one(db)
        .await?;
    let m = m.ok_or(anyhow!("Marker not found"))?;
    let mut am: marker_model::ActiveModel = m.into();

    if let Some(content) = payload.content {
        am.content = Set(Some(content));
    }
    if let Some(extra) = payload.extra {
        am.extra = Set(Some(serde_json::to_value(extra)?));
    }
    am.marker_creator_id = Set(payload.marker_creator_id);
    am.marker_title = Set(Some(payload.marker_title));
    am.picture = Set(payload.picture);
    am.picture_creator_id = Set(payload.picture_creator_id);
    am.position = Set(payload.position);
    am.hidden_flag = Set(payload.hidden_flag);
    if let Some(refresh_time) = payload.refresh_time {
        am.refresh_time = Set(refresh_time);
    }
    if let Some(video_path) = payload.video_path {
        am.video_path = Set(Some(video_path));
    }

    marker_model::Entity::update_safety(am)?.exec(db).await?;

    // item_list 全量替换（先删后插）：编辑表单始终携带完整 itemList，
    // 空列表视为清空全部关联。
    let existing = mil_model::Entity::find_safety()
        .filter(mil_model::Column::MarkerId.eq(payload.id))
        .all(db)
        .await?;
    for link in existing {
        mil_model::Entity::delete_safety(link.into())?
            .exec(db)
            .await?;
    }
    for (item_id, count) in parse_item_entries(&payload.item_list) {
        insert_item_link(db, payload.id, item_id, count).await?;
    }

    super::binary_doc::invalidate_doc_cache().await;
    Ok(CommonResponse::new(Ok(MarkerEmptyResponse {})))
}

/// 按 MarkerFilterRequest 的筛选条件收集命中点位 id 集合：
/// - item_id_list：marker_item_link.item_id 命中
/// - area_id_list：经 item.area_id 命中（marker 表无 area_id 列，经关联物品过滤）
/// - type_id_list：经 item_type_link.type_id 命中（item 表无 type 列）
/// - ���ε���ͬʱ����ʱȡ��������δ����ʱ���� None�����÷�����ȫ������
async fn collect_filtered_marker_ids(
    db: &sea_orm::DatabaseConnection,
    payload: &MarkerFilterRequest,
) -> Result<Option<HashSet<i64>>> {
    let mut item_ids: Option<HashSet<i64>> = None;
    if let Some(ids) = &payload.item_id_list {
        item_ids = Some(ids.iter().copied().collect());
    }
    if let Some(area_ids) = &payload.area_id_list {
        let mut area_items: HashSet<i64> = HashSet::new();
        for chunk in area_ids.chunks(1000) {
            for it in item_model::Entity::find_safety()
                .filter(item_model::Column::AreaId.is_in(chunk))
                .all(db)
                .await?
            {
                area_items.insert(it.id);
            }
        }
        item_ids = match item_ids {
            Some(mut prev) => {
                prev.retain(|id| area_items.contains(id));
                Some(prev)
            },
            None => Some(area_items),
        };
    }
    if let Some(type_ids) = &payload.type_id_list {
        let mut type_items: HashSet<i64> = HashSet::new();
        for chunk in type_ids.chunks(1000) {
            for it in itl_model::Entity::find_safety()
                .filter(itl_model::Column::TypeId.is_in(chunk))
                .all(db)
                .await?
            {
                type_items.insert(it.item_id);
            }
        }
        item_ids = match item_ids {
            Some(mut prev) => {
                prev.retain(|id| type_items.contains(id));
                Some(prev)
            },
            None => Some(type_items),
        };
    }
    let Some(item_ids) = item_ids else {
        return Ok(None);
    };
    let mut marker_ids: HashSet<i64> = HashSet::new();
    let item_vec: Vec<i64> = item_ids.into_iter().collect();
    for chunk in item_vec.chunks(1000) {
        for l in mil_model::Entity::find_safety()
            .filter(mil_model::Column::ItemId.is_in(chunk))
            .all(db)
            .await?
        {
            marker_ids.insert(l.marker_id);
        }
    }
    Ok(Some(marker_ids))
}

pub async fn do_get_id(
    _auth: AuthInfo,
    payload: MarkerFilterRequest,
) -> Result<CommonResponse<Vec<i64>>> {
    let db = &DB_CONN.wait().pg_conn;

    // itemIdList / areaIdList / typeIdList 任一命中即按条件过滤（多条件取交集）
    if let Some(ids) = collect_filtered_marker_ids(db, &payload).await? {
        let mut v: Vec<i64> = ids.into_iter().collect();
        v.sort_unstable();
        return Ok(CommonResponse::new(Ok(v)));
    }

    // 回退：返回所有 marker id
    let total_list = marker_model::Entity::find_safety()
        .select_only()
        .column(marker_model::Column::Id)
        .all(db)
        .await?;
    let ids: Vec<i64> = total_list.into_iter().map(|m| m.id).collect();
    Ok(CommonResponse::new(Ok(ids)))
}

pub async fn do_get_list_by_info(
    _auth: AuthInfo,
    payload: MarkerFilterRequest,
) -> Result<CommonResponse<Vec<MarkerVO>>> {
    let db = &DB_CONN.wait().pg_conn;

    // 重用 do_get_id 的逻辑获取 id 列表，然后查询模型
    let ids = match collect_filtered_marker_ids(db, &payload).await? {
        Some(ids) => {
            let mut v: Vec<i64> = ids.into_iter().collect();
            v.sort_unstable();
            v
        },
        None => marker_model::Entity::find_safety()
            .all(db)
            .await?
            .into_iter()
            .map(|m| m.id)
            .collect(),
    };

    if ids.is_empty() {
        return Ok(CommonResponse::new(Ok(vec![])));
    }

    // Chunk the IDs to avoid exceeding sqlx's 65535 parameter limit
    // (104K markers would create 104K bind params).
    let item_map = marker_item_map(db, &ids).await?;
    let linkage_map = marker_linkage_map(db, &ids).await?;
    let mut user_ids: HashSet<i64> = HashSet::new();
    let mut arr = Vec::new();
    for chunk in ids.chunks(10000) {
        let items = marker_model::Entity::find_safety()
            .filter(marker_model::Column::Id.is_in(chunk))
            .all(db)
            .await?;
        user_ids.extend(items.iter().filter_map(|m| m.creator_id));
        user_ids.extend(items.iter().filter_map(|m| m.updater_id));
        for it in items {
            arr.push(model_to_vo(it, &item_map, Some(&linkage_map)));
        }
    }
    let users = super::sys_user_map(db, &user_ids).await?;
    Ok(CommonResponse::new(Ok(arr)).with_users(users))
}

pub async fn do_get_list_by_id(
    _auth: AuthInfo,
    payload: Vec<i64>,
) -> Result<CommonResponse<Vec<MarkerVO>>> {
    const MAX_BATCH: usize = 1000;
    if payload.len() > MAX_BATCH {
        {
            return Err(anyhow!(
                "batch too large: {} > {}",
                payload.len(),
                MAX_BATCH
            ));
        }
    }

    let db = &DB_CONN.wait().pg_conn;
    if payload.is_empty() {
        return Ok(CommonResponse::new(Ok(vec![])));
    }
    let items = marker_model::Entity::find_safety()
        .filter(marker_model::Column::Id.is_in(payload))
        .all(db)
        .await?;
    let ids: Vec<i64> = items.iter().map(|m| m.id).collect();
    let item_map = marker_item_map(db, &ids).await?;
    let linkage_map = marker_linkage_map(db, &ids).await?;
    let mut user_ids: HashSet<i64> = items.iter().filter_map(|m| m.creator_id).collect();
    user_ids.extend(items.iter().filter_map(|m| m.updater_id));
    let mut arr = Vec::with_capacity(items.len());
    for it in items {
        arr.push(model_to_vo(it, &item_map, Some(&linkage_map)));
    }
    let users = super::sys_user_map(db, &user_ids).await?;
    Ok(CommonResponse::new(Ok(arr)).with_users(users))
}

pub async fn do_get_page(
    _auth: AuthInfo,
    payload: Pagination,
) -> Result<CommonResponse<MarkerListResponse>> {
    let db = &DB_CONN.wait().pg_conn;

    let size = payload.size.unwrap_or(10) as u64;
    let current = payload.current.unwrap_or(1);
    let offset = (current.saturating_sub(1) as u64).saturating_mul(size);

    let query = marker_model::Entity::find_safety();
    let total = query.clone().count(db).await?;
    let items = query.limit(size).offset(offset).all(db).await?;

    let ids: Vec<i64> = items.iter().map(|m| m.id).collect();
    let item_map = marker_item_map(db, &ids).await?;
    let linkage_map = marker_linkage_map(db, &ids).await?;
    let mut user_ids: HashSet<i64> = items.iter().filter_map(|m| m.creator_id).collect();
    user_ids.extend(items.iter().filter_map(|m| m.updater_id));
    let mut arr = Vec::with_capacity(items.len());
    for it in items {
        arr.push(model_to_vo(it, &item_map, Some(&linkage_map)));
    }
    let users = super::sys_user_map(db, &user_ids).await?;
    Ok(CommonResponse::new(Ok(MarkerListResponse {
        total: total as usize,
        size: Some(size as i64),
        items: arr,
    }))
    .with_users(users))
}

pub async fn do_delete(auth: AuthInfo, id: i64) -> Result<CommonResponse<MarkerEmptyResponse>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    let m = marker_model::Entity::find_safety_by_id(id).one(db).await?;
    let m = m.ok_or(anyhow!("Marker not found"))?;
    let mut am: marker_model::ActiveModel = m.into();
    am.del_flag = Set(true);
    marker_model::Entity::delete_safety(am)?.exec(db).await?;

    // 级联软删该 marker 的 item 关联
    let item_links = mil_model::Entity::find_safety()
        .filter(mil_model::Column::MarkerId.eq(id))
        .all(db)
        .await?;
    for link in item_links {
        mil_model::Entity::delete_safety(link.into())?
            .exec(db)
            .await?;
    }

    // 级联软删该 marker 参与的连线（from_id 或 to_id 命中）
    let linkages = linkage_model::Entity::find_safety()
        .filter(
            sea_orm::Condition::any()
                .add(linkage_model::Column::FromId.eq(id))
                .add(linkage_model::Column::ToId.eq(id)),
        )
        .all(db)
        .await?;
    for linkage in linkages {
        linkage_model::Entity::delete_safety(linkage.into())?
            .exec(db)
            .await?;
    }

    super::binary_doc::invalidate_doc_cache().await;
    Ok(CommonResponse::new(Ok(MarkerEmptyResponse {})))
}
