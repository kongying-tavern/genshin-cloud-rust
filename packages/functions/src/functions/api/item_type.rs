use anyhow::{Result, anyhow};

use sea_orm::{
    ActiveValue::{NotSet, Set},
    ExprTrait, QueryFilter, QuerySelect,
    prelude::*,
};

use _database::{
    DB_CONN, models::item::item_type as item_type_model, models::item::item_type_link as link_model,
};
use _utils::models::common::EmptyResponse;
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        item_type::{
            ItemTypeAddRequest, ItemTypeAllResponse, ItemTypeListRequest, ItemTypeListResponse,
            ItemTypeUpdateData, ItemTypeVO,
        },
        wrapper::CommonResponse,
    },
};

/// 重算某类型的 `is_final`：无未删除子级 → true（末端），有子级 → false。
/// 仅在实际值变化时写库。type_id <= 0（根占位）直接跳过。
async fn refresh_is_final(db: &sea_orm::DatabaseConnection, type_id: i64) -> Result<()> {
    if type_id <= 0 {
        return Ok(());
    }
    if let Some(parent) = item_type_model::Entity::find_safety_by_id(type_id)
        .one(db)
        .await?
    {
        let remain = item_type_model::Entity::find_safety()
            .filter(item_type_model::Column::ParentId.eq(parent.id))
            .count(db)
            .await?;
        let is_final = remain == 0;
        if parent.is_final != is_final {
            let mut pam: item_type_model::ActiveModel = parent.into();
            pam.is_final = Set(is_final);
            item_type_model::Entity::update_safety(pam)?
                .exec(db)
                .await?;
        }
    }
    Ok(())
}

// 更新类型
pub async fn do_update(
    auth: AuthInfo,
    payload: ItemTypeUpdateData,
) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    // Java updateItemType：禁止自身父子（同文案）
    if payload.id == payload.parent_id {
        return Err(anyhow!("物品类型ID不允许与父ID相同，会造成自身父子"));
    }
    let item = item_type_model::Entity::find_safety_by_id(payload.id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let Some(item) = item else {
        return Ok(CommonResponse::new(Ok(false)));
    };

    // 父级变化时联动新旧父级的末端标志（Java updateItemTypeIsFinal 语义）
    if item.parent_id != payload.parent_id {
        set_parent_is_final(&DB_CONN.wait().pg_conn, payload.parent_id, false).await;
        recalc_parent_is_final(&DB_CONN.wait().pg_conn, item.parent_id, true).await;
    }

    let mut am: item_type_model::ActiveModel = item.into();
    // 审计字段：修改时设置 update 组（update_time 由 before_save 钩子刷新）
    am.updater_id = Set(Some(auth.info.id));

    // icon_tag -> icon_id
    am.icon_id = Set(resolve_icon_id(payload.icon_id, payload.icon_tag.as_deref()).await?);

    if let Some(name) = payload.name {
        am.name = Set(name);
    }
    am.content = Set(payload.content);
    am.parent_id = Set(payload.parent_id);
    // Java updateItemTypeIsFinal(entity)：无子级才是末端（请求值被重算覆盖）
    let children = item_type_model::Entity::find_safety()
        .filter(item_type_model::Column::ParentId.eq(payload.id))
        .count(&DB_CONN.wait().pg_conn)
        .await?;
    am.is_final = Set(children == 0);
    am.hidden_flag = Set(payload.hidden_flag);
    if let Some(si) = payload.sort_index {
        am.sort_index = Set(si.clamp(i32::MIN as i64, i32::MAX as i64) as i32);
    }

    item_type_model::Entity::update_safety(am)?
        .exec(&DB_CONN.wait().pg_conn)
        .await?;
    super::binary_doc::invalidate_item_doc_cache().await;
    super::super::ws::ws_broadcast_debounced(
        "ItemBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(CommonResponse::new(Ok(true)))
}

// 将一组类型（typeId 列表）移动到目标类型下（更新 item_type.parent_id）
pub async fn do_move_to_target(
    auth: AuthInfo,
    target_type_id: i64,
    payload: Vec<i64>,
) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    const MAX_BATCH: usize = 1000;
    if payload.len() > MAX_BATCH {
        return Err(anyhow!(
            "batch too large: {} > {}",
            payload.len(),
            MAX_BATCH
        ));
    }
    let db = &DB_CONN.wait().pg_conn;
    // Java moveItemType：目标类型不得在移动集合内（防自身父子，同文案）
    if payload.contains(&target_type_id) {
        return Err(anyhow!("物品类型ID不允许与父ID相同，会造成自身父子"));
    }
    // 校验目标类型存在
    if item_type_model::Entity::find_safety_by_id(target_type_id)
        .one(db)
        .await?
        .is_none()
    {
        return Err(anyhow!("ItemType not found: {target_type_id}"));
    }
    // 记录被移动类型的原父级，移动后重算 is_final。
    // Java selectList(in ids)：集合中不存在的类型静默跳过。
    let mut old_parents: Vec<i64> = Vec::new();
    for type_id in payload {
        let Some(item) = item_type_model::Entity::find_safety_by_id(type_id)
            .one(db)
            .await?
        else {
            continue;
        };
        if item.parent_id != target_type_id {
            old_parents.push(item.parent_id);
            let mut am: item_type_model::ActiveModel = item.into();
            am.parent_id = Set(target_type_id);
            // 审计字段：修改时设置 update 组（update_time 由 before_save 钩子刷新）
            am.updater_id = Set(Some(auth.info.id));
            item_type_model::Entity::update_safety(am)?.exec(db).await?;
        }
    }
    refresh_is_final(db, target_type_id).await?;
    for p in old_parents {
        refresh_is_final(db, p).await?;
    }
    super::binary_doc::invalidate_item_doc_cache().await;
    super::super::ws::ws_broadcast_debounced(
        "ItemBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(CommonResponse::new(Ok(true)))
}

/// 父级存在（id > 0）时直接设置 isFinal（Java updateItemTypeIsFinal）。
async fn set_parent_is_final(db: &sea_orm::DatabaseConnection, parent_id: i64, is_final: bool) {
    if parent_id <= 0 {
        return;
    }
    let _: Result<()> = async {
        let Some(mut am): Option<item_type_model::ActiveModel> =
            item_type_model::Entity::find_safety_by_id(parent_id)
                .one(db)
                .await?
                .map(|m| m.into())
        else {
            return Ok(());
        };
        am.is_final = Set(is_final);
        item_type_model::Entity::update_safety(am)?.exec(db).await?;
        Ok(())
    }
    .await;
}

/// 父级 isFinal 重算（Java recalculateItemTypeIsFinal）。
async fn recalc_parent_is_final(
    db: &sea_orm::DatabaseConnection,
    parent_id: i64,
    before_modify: bool,
) {
    if parent_id == 0 {
        return;
    }
    let count = item_type_model::Entity::find_safety()
        .filter(item_type_model::Column::ParentId.eq(parent_id))
        .count(db)
        .await
        .unwrap_or(0);
    let target = if before_modify {
        count == 1
    } else {
        count == 0
    };
    set_parent_is_final(db, parent_id, target).await;
}

pub async fn do_get_list(
    auth: AuthInfo,
    self_flag: bool,
    payload: ItemTypeListRequest,
) -> Result<CommonResponse<ItemTypeListResponse>> {
    let db = &DB_CONN.wait().pg_conn;
    // 可见性（Java listItemType 的 hiddenFlagList）：按调用者角色过滤。
    let allowed = _utils::types::allowed_hidden_flags(auth.info.role_id);
    let mut query = item_type_model::Entity::find_safety()
        .filter(item_type_model::Column::HiddenFlag.is_in(allowed));
    // self=1 查询子级（前端唯一用法，body typeIdList: [-1] 取根 / [nodeId] 取子级）；
    // self=0 按 Java 语义查询自身（typeIdList 含 -1 不过滤，否则 id IN typeIdList）
    // Java listItemType 空 typeIdList 语义：self=0（查自身）→ 空页；self=1
    //（查子级）→ 根分类（parent IN [-1]）。均不做全量回退。
    let type_list = payload.type_id_list.clone().unwrap_or_default();
    if type_list.is_empty() {
        if !self_flag {
            return Ok(CommonResponse::new(Ok(ItemTypeListResponse {
                total: 0,
                items: vec![],
                size: Ord::min(payload.page.size.unwrap_or(10), 200) as i64,
            })));
        }
        query = query.filter(
            sea_orm::Condition::any()
                .add(item_type_model::Column::ParentId.eq(-1))
                .add(
                    Expr::col(item_type_model::Column::ParentId)
                        .equals(item_type_model::Column::Id),
                ),
        );
    } else if self_flag {
        // typeIdList 语义（物品类型树）：[-1] 返回根类型（parent_id=-1 或自指顶层），
        // [nodeId] 返回其子级（parent_id IN typeIdList）
        if type_list.contains(&-1) {
            query = query.filter(
                sea_orm::Condition::any()
                    .add(item_type_model::Column::ParentId.eq(-1))
                    .add(
                        Expr::col(item_type_model::Column::ParentId)
                            .equals(item_type_model::Column::Id),
                    ),
            );
        } else {
            query = query.filter(item_type_model::Column::ParentId.is_in(type_list));
        }
    } else if !type_list.contains(&-1) {
        query = query.filter(item_type_model::Column::Id.is_in(type_list));
    }

    let size_raw = payload.page.size.unwrap_or(10);
    let size: u64 = (if size_raw > 200 { 200 } else { size_raw }) as u64;
    let current = payload.page.current.unwrap_or(1);
    let offset = (current.saturating_sub(1) as u64).saturating_mul(size);

    let total = query.clone().count(db).await?;
    let icon_tag_map = super::icon::icon_tag_map(db).await?;
    let items = query.limit(size).offset(offset).all(db).await?;

    let items_val: Vec<ItemTypeVO> = items
        .into_iter()
        .map(|i| ItemTypeVO {
            version: i.version,
            id: i.id,
            create_time: i.create_time.and_utc().timestamp_millis() as f64,
            update_time: i
                .update_time
                .map(|dt| dt.and_utc().timestamp_millis() as f64),
            creator_id: i.creator_id,
            updater_id: i.updater_id,
            name: i.name,
            icon_tag: Some(icon_tag_map.get(&i.icon_id).cloned().unwrap_or_default()),
            icon_id: i.icon_id,
            content: i.content,
            parent_id: i.parent_id,
            is_final: i.is_final,
            hidden_flag: i.hidden_flag,
            sort_index: i.sort_index,
        })
        .collect();
    let body = ItemTypeListResponse {
        total: total as i64,
        items: items_val,
        size: size as i64,
    };
    Ok(CommonResponse::new(Ok(body)))
}

pub async fn do_get_list_all(auth: AuthInfo) -> Result<CommonResponse<ItemTypeAllResponse>> {
    let icon_tag_map = super::icon::icon_tag_map(&DB_CONN.wait().pg_conn).await?;
    // 可见性：list_all 与 list 同口径过滤。
    let allowed = _utils::types::allowed_hidden_flags(auth.info.role_id);
    let items = item_type_model::Entity::find_safety()
        .filter(item_type_model::Column::HiddenFlag.is_in(allowed))
        .all(&DB_CONN.wait().pg_conn)
        .await?;
    let vec = items
        .into_iter()
        .map(|i| ItemTypeVO {
            version: i.version,
            id: i.id,
            create_time: i.create_time.and_utc().timestamp_millis() as f64,
            update_time: i
                .update_time
                .map(|dt| dt.and_utc().timestamp_millis() as f64),
            creator_id: i.creator_id,
            updater_id: i.updater_id,
            name: i.name,
            icon_tag: Some(icon_tag_map.get(&i.icon_id).cloned().unwrap_or_default()),
            icon_id: i.icon_id,
            content: i.content,
            parent_id: i.parent_id,
            is_final: i.is_final,
            hidden_flag: i.hidden_flag,
            sort_index: i.sort_index,
        })
        .collect();
    Ok(CommonResponse::new(Ok(ItemTypeAllResponse(vec))))
}

// 逻辑删除类型
pub async fn do_delete(auth: AuthInfo, id: i64) -> Result<CommonResponse<EmptyResponse>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;

    // 一次性加载全部未删除类型，BFS 收集自身及全部后代
    let all = item_type_model::Entity::find_safety().all(db).await?;
    let root = all
        .iter()
        .find(|t| t.id == id)
        .ok_or(anyhow!("ItemType not found"))?;
    let root_parent_id = root.parent_id;
    let mut children: std::collections::HashMap<i64, Vec<i64>> = std::collections::HashMap::new();
    for t in &all {
        children.entry(t.parent_id).or_default().push(t.id);
    }
    let mut to_delete: Vec<i64> = Vec::new();
    let mut queue: Vec<i64> = vec![id];
    // visited 防环：parent_id 由客户端控制，可构造 A.parent=B、B.parent=A 的环
    // （或自指 parent_id=id），不加去重会无限遍历/无限写库。
    let mut visited: std::collections::HashSet<i64> = std::collections::HashSet::new();
    visited.insert(id);
    while let Some(cur) = queue.pop() {
        to_delete.push(cur);
        if let Some(cs) = children.get(&cur) {
            for c in cs {
                if visited.insert(*c) {
                    queue.push(*c);
                }
            }
        }
    }

    // 软删自身与所有后代
    for tid in &to_delete {
        if let Some(model) = all.iter().find(|t| t.id == *tid) {
            let mut am: item_type_model::ActiveModel = model.clone().into();
            am.del_flag = Set(true);
            // 审计字段：软删也是修改，设置 update 组
            am.updater_id = Set(Some(auth.info.id));
            item_type_model::Entity::delete_safety(am)?.exec(db).await?;
        }
    }

    // 清理这些类型下的 item_type_link（软删）
    link_model::Entity::update_many()
        .col_expr(
            link_model::Column::DelFlag,
            sea_orm::sea_query::Expr::value(true),
        )
        .filter(link_model::Column::TypeId.is_in(to_delete.iter().copied()))
        .exec(db)
        .await?;

    // is_final 重算：被删类型的父级若再无子级，恢复为末端类型
    refresh_is_final(db, root_parent_id).await?;
    super::binary_doc::invalidate_item_doc_cache().await;
    super::super::ws::ws_broadcast_debounced(
        "ItemBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

// 新增类型
pub async fn do_add(auth: AuthInfo, payload: ItemTypeAddRequest) -> Result<CommonResponse<i64>> {
    auth.require_non_anonymous()?;
    let now = chrono::Utc::now().naive_utc();
    // name 在逻辑上为必填
    let name = payload.name.ok_or(anyhow!("name required"))?;

    let sort_index = payload
        .sort_index
        .unwrap_or(0)
        .clamp(i32::MIN as i64, i32::MAX as i64) as i32;

    let icon_id = resolve_icon_id(payload.icon_id, payload.icon_tag.as_deref()).await?;

    // 前端不传 isFinal（serde(default) 为 false）时：
    // 有父级（parent_id > 0）→ 叶子类型 is_final=true；无父级 → false
    let is_final = payload.is_final || payload.parent_id > 0;

    let active = item_type_model::ActiveModel {
        version: Set(0),
        id: NotSet,
        // 审计字段：新增时 create/update 两组全部设置
        create_time: Set(now),
        update_time: Set(Some(now)),
        creator_id: Set(Some(auth.info.id)),
        updater_id: Set(Some(auth.info.id)),
        del_flag: Set(false),

        icon_id: Set(icon_id),
        name: Set(name),
        content: Set(payload.content),
        parent_id: Set(payload.parent_id),
        is_final: Set(is_final),
        hidden_flag: Set(payload.hidden_flag),
        sort_index: Set(sort_index),
    };

    let res = active.insert(&DB_CONN.wait().pg_conn).await?;
    // 父级新增子级后不再是末端类型
    refresh_is_final(&DB_CONN.wait().pg_conn, payload.parent_id).await?;
    super::binary_doc::invalidate_item_doc_cache().await;
    super::super::ws::ws_broadcast_debounced(
        "ItemBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(CommonResponse::new(Ok(res.id)))
}

/// 前端以 `iconTag`（tag 表标签名）而非 `iconId` 提交图标：
/// iconId 为 0 且提供了 iconTag 时，按 tag 名查 tag 表得到 icon_id；
/// 查不到则回退 0（不强制失败，保持与旧行为一致）。
async fn resolve_icon_id(icon_id: i64, icon_tag: Option<&str>) -> Result<i64> {
    if icon_id != 0 {
        return Ok(icon_id);
    }
    let Some(tag) = icon_tag.filter(|t| !t.is_empty()) else {
        return Ok(0);
    };
    Ok(super::icon::icon_id_by_tag(&DB_CONN.wait().pg_conn, tag)
        .await?
        .unwrap_or(0))
}
