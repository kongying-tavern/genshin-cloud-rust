use anyhow::Result;

use sea_orm::{
    ActiveValue::{NotSet, Set},
    QueryFilter,
    prelude::*,
};

use _database::{DB_CONN, models::marker::marker_linkage as linkage_model};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        marker_link::{
            MarkerLinkDeleteRequest, MarkerLinkGraphRequest, MarkerLinkListRequest, MarkerLinkVO,
            MarkerLinkage,
        },
        wrapper::CommonResponse,
    },
};

/// 将数据库模型转换为前端 VO（字段与前端 `MarkerLinkageVo` 对齐）
pub(super) fn model_to_vo(it: linkage_model::Model) -> MarkerLinkVO {
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

// Upsert 标记连接：一次请求生成一个新的关联组 ID（与 Java 实现一致），
// 传入 id > 0 -> 更新该记录并划入新组；否则插入新记录。返回新关联组 ID。
pub async fn do_link(
    auth: AuthInfo,
    payload: Vec<MarkerLinkage>,
) -> Result<CommonResponse<String>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    let group_id = uuid::Uuid::new_v4().simple().to_string();
    // 受影响组与点位（对齐 Java `LinkChangeVo`：broadcast MarkerLinked 的 data）
    let mut affected_groups: Vec<String> = vec![group_id.clone()];
    let mut affected_markers: Vec<i64> = Vec::new();
    let mut collect_markers = |id: i64| {
        if id > 0 && !affected_markers.contains(&id) {
            affected_markers.push(id);
        }
    };
    for p in payload {
        if let Some(id) = p.id
            && id > 0
        {
            // 尝试更新
            if let Some(existing) = linkage_model::Entity::find_safety_by_id(id).one(db).await? {
                if !existing.group_id.is_empty() && !affected_groups.contains(&existing.group_id) {
                    affected_groups.push(existing.group_id.clone());
                }
                collect_markers(existing.from_id);
                collect_markers(existing.to_id);
                let mut am: linkage_model::ActiveModel = existing.into();
                am.group_id = Set(group_id.clone());
                am.from_id = Set(p.from_id);
                am.to_id = Set(p.to_id);
                if let Some(action) = p.link_action {
                    am.link_action = Set(action);
                }
                if let Some(reverse) = p.link_reverse {
                    am.link_reverse = Set(reverse);
                }
                am.path = Set(p.path.and_then(|v| serde_json::to_value(v).ok()));
                linkage_model::Entity::update_safety(am)?.exec(db).await?;
                continue;
            }
        }

        collect_markers(p.from_id);
        collect_markers(p.to_id);
        // 插入新记录
        let now = chrono::Utc::now().naive_utc();
        let active = linkage_model::ActiveModel {
            version: Set(0),
            id: NotSet,
            create_time: Set(now),
            update_time: Set(None),
            creator_id: Set(None),
            updater_id: Set(None),
            del_flag: Set(false),

            group_id: Set(group_id.clone()),
            from_id: Set(p.from_id),
            to_id: Set(p.to_id),
            link_action: Set(p
                .link_action
                .unwrap_or(_utils::types::MarkerLinkageLinkAction::Trigger)),
            link_reverse: Set(p.link_reverse.unwrap_or(false)),
            path: Set(p.path.and_then(|v| serde_json::to_value(v).ok())),
            extra: Set(None),
        };
        active.insert(db).await?;
    }
    super::binary_doc::invalidate_doc_cache().await;
    super::super::ws::ws_broadcast(
        "MarkerLinked",
        serde_json::json!({
            "groups": affected_groups,
            "markers": affected_markers
        }),
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
    let items = linkage_model::Entity::find_safety()
        .filter(linkage_model::Column::GroupId.is_in(payload.group_ids))
        .all(db)
        .await?;
    // 按 group_id 分组返回，前端期望 `Record<string, MarkerLinkageVo[]>`
    let mut map: std::collections::HashMap<String, Vec<MarkerLinkVO>> =
        std::collections::HashMap::new();
    for it in items {
        let vo = model_to_vo(it);
        let group_id = vo.group_id.clone().unwrap_or_default();
        map.entry(group_id).or_default().push(vo);
    }
    Ok(CommonResponse::new(Ok(serde_json::to_value(map)?)))
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
    // 每个 groupId 返回一个 GraphVo 结构。前端类型（markerLink.ts GraphVo）声明
    // relations 为 `Record<string, string[]>`，但该接口前端无调用方；做最小对齐：
    // relations 改为该组 link 的 id 字符串列表，relRefs/pathRefs 无更丰富的图数据，置空数组。
    let mut map: std::collections::HashMap<String, serde_json::Value> =
        std::collections::HashMap::new();
    for g in payload.group_ids {
        let items = linkage_model::Entity::find_safety()
            .filter(linkage_model::Column::GroupId.eq(g.clone()))
            .all(db)
            .await?;
        let relations: Vec<String> = items.into_iter().map(|it| it.id.to_string()).collect();
        map.insert(
            g,
            serde_json::json!({
                "relations": relations,
                "relRefs": [],
                "pathRefs": []
            }),
        );
    }
    Ok(CommonResponse::new(Ok(serde_json::to_value(map)?)))
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
    Ok(CommonResponse::new(Ok(serde_json::json!({
        "groups": groups,
        "markers": markers
    }))))
}
