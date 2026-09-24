use anyhow::{Result, anyhow};

// serde_json not needed after concrete response conversion

use sea_orm::{
    ActiveValue::{NotSet, Set},
    ExprTrait, QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
    prelude::*,
};

use _database::{
    DB_CONN, models::area::item_area_public as common_item_model, models::item::item as item_model,
    models::item::item_type as item_type_model, models::item::item_type_link as link_model,
    models::marker::marker_item_link as marker_link_model,
};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        item::{
            ItemAddRequest, ItemFilterRequest, ItemListResponse, ItemSort, ItemUpdateData, ItemVO,
        },
        wrapper::CommonResponse,
    },
};

/// 转义 LIKE 通配符（% _ \），防止输入被当作模糊匹配通配符放大（PG 默认 ESCAPE 为反斜杠）。
fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// 全部 `item_type_link` 的 item_id → type_id 列表映射。
/// 前端按 `typeIdList` 过滤/分组物品，`ItemVO` 必须携带该字段。
pub(crate) async fn type_id_map(
    db: &sea_orm::DatabaseConnection,
) -> Result<std::collections::HashMap<i64, Vec<i64>>> {
    let links = link_model::Entity::find_safety().all(db).await?;
    let mut map: std::collections::HashMap<i64, Vec<i64>> = std::collections::HashMap::new();
    for l in links {
        map.entry(l.item_id).or_default().push(l.type_id);
    }
    Ok(map)
}

/// 全部 `marker_item_link` 的 item_id → 关联数（Java `ItemVo.count`）。
pub(crate) async fn marker_count_map(
    db: &sea_orm::DatabaseConnection,
) -> Result<std::collections::HashMap<i64, i64>> {
    let links = marker_link_model::Entity::find_safety().all(db).await?;
    let mut map: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
    for l in links {
        *map.entry(l.item_id).or_default() += 1;
    }
    Ok(map)
}

pub(crate) fn item_to_vo(
    it: &item_model::Model,
    type_map: &std::collections::HashMap<i64, Vec<i64>>,
    icon_tag_map: &std::collections::HashMap<i64, String>,
    count_map: &std::collections::HashMap<i64, i64>,
) -> ItemVO {
    ItemVO {
        version: it.version,
        id: it.id,
        create_time: it.create_time.and_utc().timestamp_millis() as f64,
        update_time: it
            .update_time
            .map(|dt| dt.and_utc().timestamp_millis() as f64),
        creator_id: it.creator_id,
        updater_id: it.updater_id,
        name: it.name.clone(),
        area_id: it.area_id,
        default_refresh_time: it.default_refresh_time,
        default_content: it.default_content.clone(),
        default_count: it.default_count,
        icon_tag: Some(icon_tag_map.get(&it.icon_id).cloned().unwrap_or_default()),
        icon_id: it.icon_id,
        icon_style_type: it.icon_style_type,
        hidden_flag: it.hidden_flag,
        sort_index: it.sort_index,
        special_flag: it.special_flag.map(|v| v as i64),
        type_id_list: type_map.get(&it.id).cloned().unwrap_or_default(),
        count: count_map.get(&it.id).copied(),
        count_split: None,
    }
}

// 批量更新物品（支持单条或多条）
pub async fn do_update(
    auth: AuthInfo,
    edit_same: bool,
    payload: Vec<ItemUpdateData>,
) -> Result<CommonResponse<bool>> {
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
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    for p in payload {
        // editSame=1：按 name 找到全部同名物品一起更新（Java 语义）
        let target_ids: Vec<i64> = if edit_same {
            item_model::Entity::find_safety()
                .filter(item_model::Column::Name.eq(p.name.clone()))
                .all(db)
                .await?
                .into_iter()
                .map(|m| m.id)
                .collect()
        } else {
            vec![p.id]
        };
        if target_ids.is_empty() {
            // editSame=1 按名称查不到同名物品（名称已改名/新建）：视为空成功，不报错
            if edit_same {
                continue;
            }
            return Err(anyhow!("Item not found"));
        }
        for id in target_ids {
            update_one(db, auth.info.id, id, &p).await?;
        }
    }
    super::binary_doc::invalidate_item_doc_cache().await;
    super::super::ws::ws_broadcast_debounced(
        "ItemBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(CommonResponse::new(Ok(true)))
}

/// 更新单个物品及其类型关联（`typeIdList` 全量替换）。
async fn update_one(
    db: &sea_orm::DatabaseConnection,
    operator_id: i64,
    id: i64,
    p: &ItemUpdateData,
) -> Result<()> {
    let item = item_model::Entity::find_safety_by_id(id).one(db).await?;
    let item = item.ok_or(anyhow!("Item not found"))?;
    let mut am: item_model::ActiveModel = item.into();
    // 审计字段：修改时设置 update 组（update_time 由 before_save 钩子刷新）
    am.updater_id = Set(Some(operator_id));

    am.icon_id = Set(resolve_icon_id(p.icon_id, p.icon_tag.as_deref()).await?);

    am.name = Set(p.name.clone());
    am.area_id = Set(p.area_id);
    am.default_content = Set(p.default_content.clone());
    if let Some(count) = p.default_count {
        am.default_count = Set(count.clamp(i32::MIN as i64, i32::MAX as i64) as i32);
    }
    am.default_refresh_time = Set(p.default_refresh_time.unwrap_or(0));
    if let Some(style) = p.icon_style_type {
        am.icon_style_type = Set(style);
    }
    am.hidden_flag = Set(p.hidden_flag);
    if let Some(si) = p.sort_index {
        am.sort_index = Set(si.clamp(i32::MIN as i64, i32::MAX as i64) as i32);
    }
    am.special_flag = Set(p
        .special_flag
        .map(|v| v.clamp(i32::MIN as i64, i32::MAX as i64) as i32));

    item_model::Entity::update_safety(am)?.exec(db).await?;

    // 类型关联：先逻辑删除旧 link，再按新 typeIdList 插入。
    // 删除+插入包事务，失败整体回滚，避免半更新状态（物品已改、类型关联丢失）。
    let txn = db.begin().await?;
    let old_links = link_model::Entity::find_safety()
        .filter(link_model::Column::ItemId.eq(id))
        .all(&txn)
        .await?;
    for link in old_links {
        let mut lam: link_model::ActiveModel = link.into();
        lam.del_flag = Set(true);
        // 审计字段：软删也是修改，设置 update 组
        lam.updater_id = Set(Some(operator_id));
        link_model::Entity::update_safety(lam)?.exec(&txn).await?;
    }
    for t in &p.type_id_list {
        let now = chrono::Utc::now().naive_utc();
        let active = link_model::ActiveModel {
            version: Set(1),
            id: NotSet,
            // 审计字段：新增时 create/update 两组全部设置
            create_time: Set(now),
            update_time: Set(Some(now)),
            creator_id: Set(Some(operator_id)),
            updater_id: Set(Some(operator_id)),
            del_flag: Set(false),

            type_id: Set(*t),
            item_id: Set(id),
        };
        active.insert(&txn).await?;
    }
    txn.commit().await?;
    Ok(())
}

// 列表（带过滤、分页、排序）
pub async fn do_get_list(
    auth: AuthInfo,
    payload: ItemFilterRequest,
) -> Result<CommonResponse<ItemListResponse>> {
    let db = &DB_CONN.wait().pg_conn;
    // 可见性（Java listItem 的 hiddenFlagList）：按调用者角色过滤。
    let allowed = _utils::types::allowed_hidden_flags(auth.info.role_id);
    let mut query =
        item_model::Entity::find_safety().filter(item_model::Column::HiddenFlag.is_in(allowed));

    if let Some(area_ids) = payload.area_id_list
        && !area_ids.is_empty()
    {
        query = query.filter(item_model::Column::AreaId.is_in(area_ids));
    }
    if let Some(name) = payload.name {
        query = query.filter(item_model::Column::Name.like(format!("%{}%", escape_like(&name))));
    }
    if let Some(sf) = payload.special_flag {
        // Java parity: special_flag is a bit-mask. param == 0 means "no special
        // flag set" (filter special_flag = 0); param > 0 means "has any of these
        // bits" (filter (special_flag & param) != 0).
        let sf = (sf as i64).clamp(i32::MIN as i64, i32::MAX as i64) as i32;
        if sf == 0 {
            query = query.filter(item_model::Column::SpecialFlag.eq(0));
        } else {
            query = query.filter(Expr::col(item_model::Column::SpecialFlag).bit_and(sf).ne(0));
        }
    }
    if let Some(type_list) = payload.type_id_list {
        // 与 link 表联表以按类型进行筛选。
        // typeIdList 有值时恒执行过滤：空命中集也必须过滤（返回空页），
        // 否则 `!item_ids.is_empty()` 守卫会丢弃过滤条件而返回全量数据。
        let ids = link_model::Entity::find_safety()
            .filter(link_model::Column::TypeId.is_in(type_list))
            .all(db)
            .await?;
        let item_ids: Vec<i64> = ids.into_iter().map(|l| l.item_id).collect();
        query = query.filter(item_model::Column::Id.is_in(item_ids));
    }

    // sortIndex 排序（前端恒传 "sortIndex-"；取首个排序条件）
    if let Some(sorts) = payload.sort
        && let Some(sort) = sorts.first()
    {
        query = match sort {
            ItemSort::SortIndexDesc => {
                query.order_by(item_model::Column::SortIndex, sea_orm::Order::Desc)
            },
            ItemSort::SortIndexAsc => {
                query.order_by(item_model::Column::SortIndex, sea_orm::Order::Asc)
            },
        };
    }

    let size_raw = payload.page.size.unwrap_or(10);
    let size: u64 = (if size_raw > 200 { 200 } else { size_raw }) as u64;
    let current = payload.page.current.unwrap_or(1);
    let offset = (current.saturating_sub(1) as u64).saturating_mul(size);

    let total = query.clone().count(db).await?;
    let icon_tag_map = super::icon::icon_tag_map(db).await?;
    let items = query.limit(size).offset(offset).all(db).await?;
    let type_map = type_id_map(db).await?;
    let count_map = marker_count_map(db).await?;
    let mut arr = Vec::with_capacity(items.len());
    for it in items {
        arr.push(item_to_vo(&it, &type_map, &icon_tag_map, &count_map));
    }
    let payload = ItemListResponse {
        total: total as i64,
        items: arr,
    };
    Ok(CommonResponse::new(Ok(payload)))
}

// 将多个物品加入到某个类型（在 link 表中插入或更新）
pub async fn do_join_type(
    auth: AuthInfo,
    type_id: i64,
    payload: Vec<i64>,
) -> Result<CommonResponse<bool>> {
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
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    // 校验类型存在（item_type 表）
    if item_type_model::Entity::find_safety_by_id(type_id)
        .one(db)
        .await?
        .is_none()
    {
        return Err(anyhow!("类型ID错误"));
    }
    // Java ItemService.joinItemsInType：全部 itemId 必须存在，否则「物品ID存在错误」
    {
        let ids: Vec<i64> = payload.clone();
        let existing: std::collections::HashSet<i64> = item_model::Entity::find_safety()
            .filter(item_model::Column::Id.is_in(ids))
            .all(db)
            .await?
            .into_iter()
            .map(|m| m.id)
            .collect();
        if payload.iter().any(|id| !existing.contains(id)) {
            return Err(anyhow!("物品ID存在错误"));
        }
    }
    for item_id in payload {
        // 该 item 已存在此 type 的 link 则跳过（追加语义，不再覆盖其他类型关联）
        let ex = link_model::Entity::find_safety()
            .filter(link_model::Column::ItemId.eq(item_id))
            .filter(link_model::Column::TypeId.eq(type_id))
            .one(db)
            .await?;
        if ex.is_some() {
            continue;
        }
        let now = chrono::Utc::now().naive_utc();
        let active = link_model::ActiveModel {
            version: Set(1),
            id: NotSet,
            // 审计字段：新增时 create/update 两组全部设置
            create_time: Set(now),
            update_time: Set(Some(now)),
            creator_id: Set(Some(auth.info.id)),
            updater_id: Set(Some(auth.info.id)),
            del_flag: Set(false),

            type_id: Set(type_id),
            item_id: Set(item_id),
        };
        active.insert(db).await?;
    }
    super::binary_doc::invalidate_item_doc_cache().await;
    super::super::ws::ws_broadcast_debounced(
        "ItemBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(CommonResponse::new(Ok(true)))
}

pub async fn do_get_list_by_id(
    auth: AuthInfo,
    payload: Vec<i64>,
) -> Result<CommonResponse<Vec<ItemVO>>> {
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
    let icon_tag_map = super::icon::icon_tag_map(db).await?;
    // 可见性：不可见 flag 的物品对调用者如同不存在。
    let allowed = _utils::types::allowed_hidden_flags(auth.info.role_id);
    let items = item_model::Entity::find_safety()
        .filter(item_model::Column::Id.is_in(payload))
        .filter(item_model::Column::HiddenFlag.is_in(allowed))
        .all(db)
        .await?;
    let type_map = type_id_map(db).await?;
    let count_map = marker_count_map(db).await?;
    let mut arr = Vec::with_capacity(items.len());
    for it in items {
        arr.push(item_to_vo(&it, &type_map, &icon_tag_map, &count_map));
    }
    Ok(CommonResponse::new(Ok(arr)))
}

pub async fn do_delete(auth: AuthInfo, id: i64) -> Result<CommonResponse<bool>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    let Some(item) = item_model::Entity::find_safety_by_id(id).one(db).await? else {
        // Java deleteItem：不存在的 id 返回 data:false（200 + R）
        return Ok(CommonResponse::new(Ok(false)));
    };

    // Java ItemService：公共物品（item_common 命中）不允许删除
    let is_common = common_item_model::Entity::find_safety()
        .filter(common_item_model::Column::ItemId.eq(id))
        .count(db)
        .await?
        > 0;
    if is_common {
        return Err(anyhow!("不允许删除公共物品"));
    }

    let mut am: item_model::ActiveModel = item.into();
    am.del_flag = Set(true);
    // 审计字段：软删也是修改，设置 update 组
    am.updater_id = Set(Some(auth.info.id));
    item_model::Entity::delete_safety(am)?.exec(db).await?;

    // 清理 item_type_link 关联（软删）
    link_model::Entity::update_many()
        .col_expr(
            link_model::Column::DelFlag,
            sea_orm::sea_query::Expr::value(true),
        )
        .filter(link_model::Column::ItemId.eq(id))
        .exec(db)
        .await?;

    // Java ItemService：同步清理 marker_item_link（Java 为物理删除，此处软删）
    marker_link_model::Entity::update_many()
        .col_expr(
            marker_link_model::Column::DelFlag,
            sea_orm::sea_query::Expr::value(true),
        )
        .filter(marker_link_model::Column::ItemId.eq(id))
        .exec(db)
        .await?;

    // Java ItemController：删除成功后同时清 item 缓存与 marker 缓存
    super::binary_doc::invalidate_doc_cache().await;
    super::super::ws::ws_broadcast_debounced(
        "ItemBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    super::super::ws::ws_broadcast_debounced(
        "MarkerBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(CommonResponse::new(Ok(true)))
}

// 复制物品到指定地区（简单实现：复制记录并关联相同类型）
// 前端契约 RListLong：返回新复制出的物品 ID 列表（data: number[]）。
pub async fn do_copy_to_area(
    auth: AuthInfo,
    area_id: i64,
    payload: Vec<i64>,
) -> Result<CommonResponse<Vec<i64>>> {
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

    auth.require_non_anonymous()?;
    let mut new_ids = Vec::with_capacity(payload.len());
    for id in payload {
        if let Some(item) = item_model::Entity::find_safety_by_id(id)
            .one(&DB_CONN.wait().pg_conn)
            .await?
        {
            let mut am: item_model::ActiveModel = item.into();
            // 复制为新行：IDENTITY 列走自增（显式 Set(0) 会在第二次复制时撞主键）
            am.id = NotSet;
            am.area_id = Set(area_id);
            // 审计字段：新增时 create/update 两组全部设置（复制人即操作者）
            let copy_now = chrono::Utc::now().naive_utc();
            am.create_time = Set(copy_now);
            am.update_time = Set(Some(copy_now));
            // 新行乐观锁从 0 起步
            am.version = Set(0);
            am.creator_id = Set(Some(auth.info.id));
            am.updater_id = Set(Some(auth.info.id));
            let res = am.insert(&DB_CONN.wait().pg_conn).await?;
            let new_id = res.id;
            // 复制类型关联
            let links = link_model::Entity::find_safety()
                .filter(link_model::Column::ItemId.eq(id))
                .all(&DB_CONN.wait().pg_conn)
                .await?;
            for l in links {
                let active = link_model::ActiveModel {
                    version: Set(1),
                    id: NotSet,
                    // 审计字段：新增时 create/update 两组全部设置
                    create_time: Set(chrono::Utc::now().naive_utc()),
                    update_time: Set(Some(chrono::Utc::now().naive_utc())),
                    creator_id: Set(Some(auth.info.id)),
                    updater_id: Set(Some(auth.info.id)),
                    del_flag: Set(false),

                    type_id: Set(l.type_id),
                    item_id: Set(new_id),
                };
                active.insert(&DB_CONN.wait().pg_conn).await?;
            }
            new_ids.push(new_id);
        }
    }
    super::binary_doc::invalidate_item_doc_cache().await;
    super::super::ws::ws_broadcast_debounced(
        "ItemBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(CommonResponse::new(Ok(new_ids)))
}

pub async fn do_add(auth: AuthInfo, payload: ItemAddRequest) -> Result<CommonResponse<i64>> {
    auth.require_non_anonymous()?;
    let now = chrono::Utc::now().naive_utc();

    let icon_id = resolve_icon_id(payload.icon_id, payload.icon_tag.as_deref()).await?;

    let active = item_model::ActiveModel {
        version: Set(1),
        id: NotSet,
        // 审计字段：新增时 create/update 两组全部设置
        create_time: Set(now),
        update_time: Set(Some(now)),
        creator_id: Set(Some(auth.info.id)),
        updater_id: Set(Some(auth.info.id)),
        del_flag: Set(false),

        name: Set(payload.name),
        area_id: Set(payload.area_id),
        default_refresh_time: Set(payload.default_refresh_time.unwrap_or(0)),
        default_content: Set(Some(payload.default_content)),
        default_count: Set(payload
            .default_count
            .clamp(i32::MIN as i64, i32::MAX as i64) as i32),
        icon_id: Set(icon_id),
        icon_style_type: Set(payload.icon_style_type),
        hidden_flag: Set(payload.hidden_flag),
        sort_index: Set(payload
            .sort_index
            .unwrap_or(0)
            .clamp(i32::MIN as i64, i32::MAX as i64) as i32),
        special_flag: Set(payload
            .special_flag
            .map(|v| v.clamp(i32::MIN as i64, i32::MAX as i64) as i32)),
    };

    let res = active.insert(&DB_CONN.wait().pg_conn).await?;
    let new_id = res.id;

    // 插入类型关联前校验类型存在（item_type 表）：
    // Java ItemService 对不存在的类型抛「类型ID错误」，不做静默跳过。
    if !payload.type_id_list.is_empty() {
        let existing: std::collections::HashSet<i64> = item_type_model::Entity::find_safety()
            .filter(item_type_model::Column::Id.is_in(payload.type_id_list.clone()))
            .all(&DB_CONN.wait().pg_conn)
            .await?
            .into_iter()
            .map(|t| t.id)
            .collect();
        if payload.type_id_list.iter().any(|t| !existing.contains(t)) {
            return Err(anyhow!("类型ID错误"));
        }
        for t in &payload.type_id_list {
            let now = chrono::Utc::now().naive_utc();
            let active = link_model::ActiveModel {
                version: Set(1),
                id: NotSet,
                // 审计字段：新增时 create/update 两组全部设置
                create_time: Set(now),
                update_time: Set(Some(now)),
                creator_id: Set(Some(auth.info.id)),
                updater_id: Set(Some(auth.info.id)),
                del_flag: Set(false),

                type_id: Set(*t),
                item_id: Set(new_id),
            };
            active.insert(&DB_CONN.wait().pg_conn).await?;
        }
    }

    super::binary_doc::invalidate_item_doc_cache().await;
    super::super::ws::ws_broadcast_debounced(
        "ItemBinaryPurged",
        serde_json::Value::Null,
        super::super::ws::PURGE_DEBOUNCE_WINDOW,
    );
    Ok(CommonResponse::new(Ok(new_id)))
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
