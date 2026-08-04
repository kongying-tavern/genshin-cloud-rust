use anyhow::{Result, anyhow};
use chrono::Utc;

use sea_orm::{
    ActiveValue::{NotSet, Set},
    QuerySelect,
    prelude::*,
};

use std::collections::HashSet;

use _database::{
    DB_CONN, models::item::item as item_model, models::marker::marker as marker_model,
    models::marker::marker_item_link as mil_model,
};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        marker::MarkerFilterRequest,
        marker::{MarkerAddRequest, MarkerItemLinkVo, MarkerTweakRequest, MarkerUpdateData},
        marker::{
            MarkerAddResponse, MarkerEmptyResponse, MarkerIdListResponse, MarkerItemsResponse,
            MarkerListResponse, MarkerVO,
        },
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
    let links = mil_model::Entity::find_safety()
        .filter(mil_model::Column::MarkerId.is_in(marker_ids))
        .all(db)
        .await?;
    let item_ids: Vec<i64> = links.iter().map(|l| l.item_id).collect();
    let mut icon_tags: std::collections::HashMap<i64, String> = std::collections::HashMap::new();
    if !item_ids.is_empty() {
        for it in item_model::Entity::find_safety()
            .filter(item_model::Column::Id.is_in(item_ids))
            .all(db)
            .await?
        {
            icon_tags.insert(it.id, it.icon_tag);
        }
    }
    for l in links {
        map.entry(l.marker_id).or_default().push(MarkerItemLinkVo {
            item_id: l.item_id,
            count: l.count,
            icon_tag: icon_tags.get(&l.item_id).cloned().unwrap_or_default(),
        });
    }
    Ok(map)
}

/// 批量调整点位数据，目前实现常用字段的替换/更新逻辑：
/// 对于复杂的 item_list 调整暂时跳过（可在后续增强）。
fn model_to_vo(
    it: marker_model::Model,
    item_map: &std::collections::HashMap<i64, Vec<MarkerItemLinkVo>>,
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
    }
}

/// camelCase marker view for the BinaryMD5 pages (the wire contract of the
/// `marker_doc` blob is the Java `MarkerVo` naming).
pub(crate) fn model_to_vo_doc(
    it: &marker_model::Model,
    item_map: &std::collections::HashMap<i64, Vec<MarkerItemLinkVo>>,
) -> MarkerVO {
    model_to_vo(it.clone(), item_map)
}

pub async fn do_tweak(
    auth: AuthInfo,
    payload: MarkerTweakRequest,
) -> Result<CommonResponse<MarkerEmptyResponse>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;

    for marker_id in payload.marker_ids.iter() {
        let m = marker_model::Entity::find_safety_by_id(*marker_id)
            .one(db)
            .await?;
        if m.is_none() {
            // 跳过缺失的标记
            continue;
        }
        let m = m.unwrap();
        let mut am: marker_model::ActiveModel = m.into();

        for tweak in payload.tweaks.iter() {
            match tweak.prop {
                _utils::models::marker::MarkerTweakConfigPropEnum::Content => {
                    if let Some(v) = &tweak.meta.replace {
                        am.content = Set(v.clone());
                    } else if let Some(_utils::models::marker::TweakValue::String(s)) =
                        &tweak.meta.value
                    {
                        // 简化处理：若值为 String -> 替换
                        am.content = Set(s.clone());
                    }
                },
                _utils::models::marker::MarkerTweakConfigPropEnum::Title => {
                    if let Some(v) = &tweak.meta.replace {
                        am.marker_title = Set(Some(v.clone()));
                    }
                },
                _utils::models::marker::MarkerTweakConfigPropEnum::Position => {
                    if let Some(v) = &tweak.meta.replace {
                        am.position = Set(v.clone());
                    }
                },
                _utils::models::marker::MarkerTweakConfigPropEnum::VideoPath => {
                    if let Some(v) = &tweak.meta.replace {
                        am.video_path = Set(Some(v.clone()));
                    }
                },
                _utils::models::marker::MarkerTweakConfigPropEnum::RefreshTime => {
                    if let Some(_v) = &tweak.meta.value
                        && let _utils::models::marker::TweakValue::Integer(i) = _v
                    {
                        am.refresh_time = Set(*i);
                    }
                },
                _utils::models::marker::MarkerTweakConfigPropEnum::Extra => {
                    if let Some(map) = &tweak.meta.map {
                        // 用序列化后的 map 完整替换 extra
                        am.extra = Set(Some(serde_json::to_value(map)?));
                    } else if let Some(_utils::models::marker::TweakValue::AnythingMap(m)) =
                        &tweak.meta.value
                    {
                        // 尝试设置任意 JSON 值
                        am.extra = Set(Some(serde_json::to_value(m)?));
                    }
                },
                _utils::models::marker::MarkerTweakConfigPropEnum::HiddenFlag => {
                    if let Some(_val) = &tweak.meta.value
                        && let _utils::models::marker::TweakValue::Integer(i) = _val
                    {
                        // HiddenFlag 是一个枚举；utils 中定义。尝试从整数转换。
                        let hf = match *i as i32 {
                            0 => _utils::types::HiddenFlag::Visible,
                            1 => _utils::types::HiddenFlag::Hidden,
                            2 => _utils::types::HiddenFlag::Spy,
                            3 => _utils::types::HiddenFlag::Suprise,
                            _ => _utils::types::HiddenFlag::Visible,
                        };
                        am.hidden_flag = Set(hf);
                    }
                },
                _utils::models::marker::MarkerTweakConfigPropEnum::ItemList => {
                    tweak_item_list(db, *marker_id, tweak).await?;
                },
            }
        }

        // 通过 ActiveModelBehavior 设置 updater 与 update_time；确保携带版本信息
        marker_model::Entity::update_safety(am)?.exec(db).await?;
    }

    Ok(CommonResponse::new(Ok(MarkerEmptyResponse {})))
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
                if let Some(id) = obj.get("id").and_then(|x| x.as_i64()) {
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
/// 支持的调整类型：Append / InsertIfAbsent / InsertOrUpdate / Merge / Update
/// （追加或更新 count）、Replace（整表替换）、RemoveLeft / RemoveRight
/// （移除列出的关联）。
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
        // Append / Prepend / InsertIfAbsent / InsertOrUpdate / Merge / Update：
        // 追加或更新 count；InsertIfAbsent 对已存在条目跳过。
        // 其余类型（Trim*/ReplaceRegex 等）对 item 列表无意义，忽略。
        MarkerTweakConfigTypeEnum::Append
        | MarkerTweakConfigTypeEnum::Prepend
        | MarkerTweakConfigTypeEnum::InsertIfAbsent
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
) -> Result<CommonResponse<MarkerAddResponse>> {
    auth.require_non_anonymous()?;
    let now = Utc::now().naive_utc();

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
        content: Set(payload.content.unwrap_or_default()),
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

    let res = active.insert(&DB_CONN.wait().pg_conn).await?;
    Ok(CommonResponse::new(Ok(MarkerAddResponse { id: res.id })))
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
        am.content = Set(content);
    }
    if let Some(extra) = payload.extra {
        am.extra = Set(Some(serde_json::to_value(extra)?));
    }
    am.marker_creator_id = Set(payload.marker_creator_id);
    am.marker_title = Set(Some(payload.marker_title));
    am.picture = Set(payload.picture);
    am.picture_creator_id = Set(payload.picture_creator_id);
    am.position = Set(payload.position);
    if let Some(refresh_time) = payload.refresh_time {
        am.refresh_time = Set(refresh_time);
    }
    if let Some(video_path) = payload.video_path {
        am.video_path = Set(Some(video_path));
    }

    marker_model::Entity::update_safety(am)?.exec(db).await?;
    Ok(CommonResponse::new(Ok(MarkerEmptyResponse {})))
}

pub async fn do_get_id(
    _auth: AuthInfo,
    payload: MarkerFilterRequest,
) -> Result<CommonResponse<MarkerIdListResponse>> {
    let db = &DB_CONN.wait().pg_conn;

    // 如果提供了 item_id_list，则从 marker_item_link 收集 marker id
    if let Some(item_ids) = payload.item_id_list {
        let links = mil_model::Entity::find_safety()
            .filter(mil_model::Column::ItemId.is_in(item_ids))
            .all(db)
            .await?;
        let ids: HashSet<i64> = links.into_iter().map(|l| l.marker_id).collect();
        let mut v: Vec<i64> = ids.into_iter().collect();
        v.sort_unstable();
        return Ok(CommonResponse::new(Ok(MarkerIdListResponse { ids: v })));
    }

    // 回退：返回所有 marker id
    let total_list = marker_model::Entity::find_safety()
        .select_only()
        .column(marker_model::Column::Id)
        .all(db)
        .await?;
    let ids: Vec<i64> = total_list.into_iter().map(|m| m.id).collect();
    let payload = MarkerIdListResponse { ids };
    Ok(CommonResponse::new(Ok(payload)))
}

pub async fn do_get_list_by_info(
    _auth: AuthInfo,
    payload: MarkerFilterRequest,
) -> Result<CommonResponse<MarkerListResponse>> {
    let db = &DB_CONN.wait().pg_conn;

    // 重用 do_get_id 的逻辑获取 id 列表，然后查询模型
    let ids = if let Some(item_ids) = payload.item_id_list {
        let links = mil_model::Entity::find_safety()
            .filter(mil_model::Column::ItemId.is_in(item_ids))
            .all(db)
            .await?;
        let ids: HashSet<i64> = links.into_iter().map(|l| l.marker_id).collect();
        let mut v: Vec<i64> = ids.into_iter().collect();
        v.sort_unstable();
        v
    } else {
        marker_model::Entity::find_safety()
            .all(db)
            .await?
            .into_iter()
            .map(|m| m.id)
            .collect()
    };

    if ids.is_empty() {
        return Ok(CommonResponse::new(Ok(MarkerListResponse {
            total: 0,
            items: vec![],
        })));
    }

    // Chunk the IDs to avoid exceeding sqlx's 65535 parameter limit
    // (104K markers would create 104K bind params).
    let item_map = marker_item_map(db, &ids).await?;
    let mut arr = Vec::new();
    for chunk in ids.chunks(10000) {
        let items = marker_model::Entity::find_safety()
            .filter(marker_model::Column::Id.is_in(chunk))
            .all(db)
            .await?;
        for it in items {
            arr.push(model_to_vo(it, &item_map));
        }
    }
    Ok(CommonResponse::new(Ok(MarkerListResponse {
        total: arr.len(),
        items: arr,
    })))
}

pub async fn do_get_list_by_id(
    _auth: AuthInfo,
    payload: Vec<i64>,
) -> Result<CommonResponse<MarkerItemsResponse>> {
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
        return Ok(CommonResponse::new(Ok(MarkerItemsResponse {
            items: vec![],
        })));
    }
    let items = marker_model::Entity::find_safety()
        .filter(marker_model::Column::Id.is_in(payload))
        .all(db)
        .await?;
    let ids: Vec<i64> = items.iter().map(|m| m.id).collect();
    let item_map = marker_item_map(db, &ids).await?;
    let mut arr = Vec::with_capacity(items.len());
    for it in items {
        arr.push(model_to_vo(it, &item_map));
    }
    Ok(CommonResponse::new(Ok(MarkerItemsResponse { items: arr })))
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
    let mut arr = Vec::with_capacity(items.len());
    for it in items {
        arr.push(model_to_vo(it, &item_map));
    }
    Ok(CommonResponse::new(Ok(MarkerListResponse {
        total: total as usize,
        items: arr,
    })))
}

pub async fn do_delete(auth: AuthInfo, id: i64) -> Result<CommonResponse<MarkerEmptyResponse>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    let m = marker_model::Entity::find_safety_by_id(id).one(db).await?;
    let m = m.ok_or(anyhow!("Marker not found"))?;
    let mut am: marker_model::ActiveModel = m.into();
    am.del_flag = Set(true);
    marker_model::Entity::delete_safety(am)?.exec(db).await?;
    Ok(CommonResponse::new(Ok(MarkerEmptyResponse {})))
}
