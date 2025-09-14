use anyhow::Result;

use sea_orm::{prelude::*, ActiveValue::Set, QueryFilter};

use _database::{models::marker::marker_linkage as linkage_model, DB_CONN};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        marker_link::{
            MarkerLinkDeleteRequest, MarkerLinkGraphRequest, MarkerLinkGraphResponse,
            MarkerLinkListRequest, MarkerLinkListResponse, MarkerLinkUpsertResult, MarkerLinkVO,
            MarkerLinkage,
        },
        wrapper::CommonResponse,
    },
};

// Upsert 标记连接：如果传入 id > 0 -> 更新该记录；否则插入新记录
pub async fn do_link(
    _auth: AuthInfo,
    payload: Vec<MarkerLinkage>,
) -> Result<CommonResponse<Vec<MarkerLinkUpsertResult>>> {
    let mut ret = Vec::with_capacity(payload.len());
    for p in payload {
        if p.id > 0 {
            // 尝试更新
            if let Some(existing) = linkage_model::Entity::find_safety_by_id(p.id)
                .one(&DB_CONN.wait().pg_conn)
                .await?
            {
                let mut am: linkage_model::ActiveModel = existing.into();
                am.from_id = Set(p.from_id);
                am.to_id = Set(p.to_id);
                if let Some(action) = p.link_action {
                    am.link_action = Set(action);
                }
                am.path = Set(p.path.map(|v| serde_json::to_value(v).ok()).flatten());
                linkage_model::Entity::update_safety(am)
                    .exec(&DB_CONN.wait().pg_conn)
                    .await?;
                ret.push(MarkerLinkUpsertResult {
                    id: p.id,
                    status: "updated".to_string(),
                });
                continue;
            }
        }

        // 插入新记录
        let now = chrono::Utc::now().naive_utc();
        let active = linkage_model::ActiveModel {
            version: Set(0),
            id: Set(0),
            create_time: Set(now),
            update_time: Set(None),
            creator_id: Set(None),
            updater_id: Set(None),
            del_flag: Set(false),

            group_id: Set(String::new()),
            from_id: Set(p.from_id),
            to_id: Set(p.to_id),
            link_action: Set(p
                .link_action
                .unwrap_or(_utils::types::MarkerLinkageLinkAction::Trigger)),
            link_reverse: Set(false),
            path: Set(p.path.and_then(|v| serde_json::to_value(v).ok())),
            extra: Set(None),
        };
        let res = active.insert(&DB_CONN.wait().pg_conn).await?;
        ret.push(MarkerLinkUpsertResult {
            id: res.id,
            status: "inserted".to_string(),
        });
    }
    Ok(CommonResponse::new(Ok(ret)))
}

pub async fn do_get_list(
    _auth: AuthInfo,
    payload: MarkerLinkListRequest,
) -> Result<CommonResponse<MarkerLinkListResponse>> {
    let db = &DB_CONN.wait().pg_conn;
    let mut out = Vec::new();
    let mut query = linkage_model::Entity::find_safety();
    if !payload.group_ids.is_empty() {
        query = query.filter(linkage_model::Column::GroupId.is_in(payload.group_ids));
    }
    let items = query.all(db).await?;
    for it in items {
        out.push(MarkerLinkVO {
            id: it.id,
            from_id: it.from_id,
            to_id: it.to_id,
            link_action: Some(it.link_action),
            path: it.path.and_then(|j| serde_json::from_value(j).ok()),
        });
    }
    Ok(CommonResponse::new(Ok(MarkerLinkListResponse(out))))
}

pub async fn do_get_graph(
    _auth: AuthInfo,
    payload: MarkerLinkGraphRequest,
) -> Result<CommonResponse<MarkerLinkGraphResponse>> {
    // 目前按 group_id 返回分组的连接列表
    let db = &DB_CONN.wait().pg_conn;
    let groups = payload.group_ids;
    let mut map: std::collections::HashMap<String, Vec<MarkerLinkVO>> =
        std::collections::HashMap::new();
    if groups.is_empty() {
        let all = linkage_model::Entity::find_safety().all(db).await?;
        let mut vec = Vec::with_capacity(all.len());
        for it in all {
            vec.push(MarkerLinkVO {
                id: it.id,
                from_id: it.from_id,
                to_id: it.to_id,
                // DB model has a non-optional `link_action`, API VO expects `Option`.
                link_action: Some(it.link_action),
                // `path` is stored as JSON in DB; try to deserialize into expected VO type.
                path: it.path.and_then(|j| serde_json::from_value(j).ok()),
            });
        }
        map.insert("all".to_string(), vec);
    } else {
        for g in groups {
            let items = linkage_model::Entity::find_safety()
                .filter(linkage_model::Column::GroupId.eq(g.clone()))
                .all(db)
                .await?;
            let mut vec = Vec::with_capacity(items.len());
            for it in items {
                vec.push(MarkerLinkVO {
                    id: it.id,
                    from_id: it.from_id,
                    to_id: it.to_id,
                    link_action: Some(it.link_action),
                    path: it.path.and_then(|j| serde_json::from_value(j).ok()),
                });
            }
            map.insert(g, vec);
        }
    }
    Ok(CommonResponse::new(Ok(MarkerLinkGraphResponse(map))))
}

pub async fn do_delete(
    _auth: AuthInfo,
    payload: MarkerLinkDeleteRequest,
) -> Result<CommonResponse<()>> {
    let db = &DB_CONN.wait().pg_conn;
    if let Some(ids) = payload.ids {
        for id in ids {
            if let Some(item) = linkage_model::Entity::find_safety_by_id(id).one(db).await? {
                let mut am: linkage_model::ActiveModel = item.into();
                am.del_flag = Set(true);
                linkage_model::Entity::delete_safety(am).exec(db).await?;
            }
        }
        return Ok(CommonResponse::new(Ok(())));
    }

    if let Some(group_ids) = payload.group_ids {
        for gid in group_ids {
            let items = linkage_model::Entity::find_safety()
                .filter(linkage_model::Column::GroupId.eq(gid))
                .all(db)
                .await?;
            for it in items {
                let mut am: linkage_model::ActiveModel = it.into();
                am.del_flag = Set(true);
                linkage_model::Entity::delete_safety(am).exec(db).await?;
            }
        }
    }
    Ok(CommonResponse::new(Ok(())))
}
