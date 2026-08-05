use anyhow::{Result, anyhow};

// serde_json not needed after concrete response conversion

use sea_orm::{
    ActiveValue::{NotSet, Set},
    ExprTrait, QueryFilter, QuerySelect,
    prelude::*,
};

use _database::{
    DB_CONN, models::item::item as item_model, models::item::item_type_link as link_model,
};
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        item::{
            CopyCountResponse, ItemAddRequest, ItemAddResponse, ItemFilterRequest,
            ItemListResponse, ItemUpdateData, ItemVO,
        },
        wrapper::CommonResponse,
    },
};

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

pub(crate) fn item_to_vo(
    it: &item_model::Model,
    type_map: &std::collections::HashMap<i64, Vec<i64>>,
    icon_tag_map: &std::collections::HashMap<i64, String>,
) -> ItemVO {
    ItemVO {
        id: it.id,
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
        special_flag: it.special_flag,
        type_id_list: type_map.get(&it.id).cloned().unwrap_or_default(),
    }
}

// 批量更新物品（支持单条或多条）
pub async fn do_update(
    auth: AuthInfo,
    _edit_same: bool,
    payload: Vec<ItemUpdateData>,
) -> Result<CommonResponse<()>> {
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
    for p in payload {
        let item = item_model::Entity::find_safety_by_id(p.id)
            .one(&DB_CONN.wait().pg_conn)
            .await?;
        let item = item.ok_or(anyhow!("Item not found"))?;
        let mut am: item_model::ActiveModel = item.into();

        am.area_id = Set(p.area_id);
        am.default_content = Set(Some(p.default_content));
        am.default_count = Set(p.default_count as i32);
        am.default_refresh_time = Set(p.default_refresh_time.unwrap_or(0));
        am.icon_id = Set(p.icon_id);
        am.icon_style_type = Set(p.icon_style_type);
        am.hidden_flag = Set(p.hidden_flag);
        if let Some(si) = p.sort_index {
            am.sort_index = Set(si as i32);
        }
        am.special_flag = Set(Some(p.special_flag as i32));

        item_model::Entity::update_safety(am)?
            .exec(&DB_CONN.wait().pg_conn)
            .await?;
    }
    Ok(CommonResponse::new(Ok(())))
}

// 列表（带过滤、分页、排序）
pub async fn do_get_list(
    _auth: AuthInfo,
    payload: ItemFilterRequest,
) -> Result<CommonResponse<ItemListResponse>> {
    let db = &DB_CONN.wait().pg_conn;
    let mut query = item_model::Entity::find_safety();

    if let Some(area_ids) = payload.area_id_list {
        query = query.filter(item_model::Column::AreaId.is_in(area_ids));
    }
    if let Some(name) = payload.name {
        query = query.filter(item_model::Column::Name.like(format!("%{}%", name)));
    }
    if let Some(sf) = payload.special_flag {
        // Java parity: special_flag is a bit-mask. param == 0 means "no special
        // flag set" (filter special_flag = 0); param > 0 means "has any of these
        // bits" (filter (special_flag & param) != 0).
        let sf = sf as i32;
        if sf == 0 {
            query = query.filter(item_model::Column::SpecialFlag.eq(0));
        } else {
            query = query.filter(Expr::col(item_model::Column::SpecialFlag).bit_and(sf).ne(0));
        }
    }
    if let Some(type_list) = payload.type_id_list {
        // 与 link 表联表以按类型进行筛选
        let ids = link_model::Entity::find_safety()
            .filter(link_model::Column::TypeId.is_in(type_list))
            .all(db)
            .await?;
        let item_ids: Vec<i64> = ids.into_iter().map(|l| l.item_id).collect();
        if !item_ids.is_empty() {
            query = query.filter(item_model::Column::Id.is_in(item_ids));
        }
    }

    let size = payload.page.size.unwrap_or(10) as u64;
    let current = payload.page.current.unwrap_or(1);
    let offset = (current.saturating_sub(1) as u64).saturating_mul(size);

    let total = query.clone().count(db).await?;
    let icon_tag_map = super::icon::icon_tag_map(db).await?;
    let items = query.limit(size).offset(offset).all(db).await?;
    let type_map = type_id_map(db).await?;
    let mut arr = Vec::with_capacity(items.len());
    for it in items {
        arr.push(item_to_vo(&it, &type_map, &icon_tag_map));
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
) -> Result<CommonResponse<()>> {
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
    for item_id in payload {
        // 查找现有 link
        let ex = link_model::Entity::find_safety()
            .filter(link_model::Column::ItemId.eq(item_id))
            .one(&DB_CONN.wait().pg_conn)
            .await?;
        if let Some(link) = ex {
            let mut lam: link_model::ActiveModel = link.into();
            lam.type_id = Set(type_id);
            link_model::Entity::update_safety(lam)?
                .exec(&DB_CONN.wait().pg_conn)
                .await?;
        } else {
            let now = chrono::Utc::now().naive_utc();
            let active = link_model::ActiveModel {
                version: Set(0),
                id: NotSet,
                create_time: Set(now),
                update_time: Set(None),
                creator_id: Set(None),
                updater_id: Set(None),
                del_flag: Set(false),

                type_id: Set(type_id),
                item_id: Set(item_id),
            };
            active.insert(&DB_CONN.wait().pg_conn).await?;
        }
    }
    Ok(CommonResponse::new(Ok(())))
}

pub async fn do_get_list_by_id(
    _auth: AuthInfo,
    payload: Vec<i64>,
) -> Result<CommonResponse<ItemListResponse>> {
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
    let items = item_model::Entity::find_safety()
        .filter(item_model::Column::Id.is_in(payload))
        .all(db)
        .await?;
    let type_map = type_id_map(db).await?;
    let mut arr = Vec::with_capacity(items.len());
    for it in items {
        arr.push(item_to_vo(&it, &type_map, &icon_tag_map));
    }
    let payload = ItemListResponse {
        total: arr.len() as i64,
        items: arr,
    };
    Ok(CommonResponse::new(Ok(payload)))
}

pub async fn do_delete(auth: AuthInfo, id: i64) -> Result<CommonResponse<()>> {
    auth.require_non_anonymous()?;
    let item = item_model::Entity::find_safety_by_id(id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow!("Item not found"))?;
    let mut am: item_model::ActiveModel = item.into();
    am.del_flag = Set(true);
    item_model::Entity::delete_safety(am)?
        .exec(&DB_CONN.wait().pg_conn)
        .await?;
    Ok(CommonResponse::new(Ok(())))
}

// 复制物品到指定地区（简单实现：复制记录并关联相同类型）
pub async fn do_copy_to_area(
    auth: AuthInfo,
    area_id: i64,
    payload: Vec<i64>,
) -> Result<CommonResponse<CopyCountResponse>> {
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
    let mut count = 0i64;
    for id in payload {
        if let Some(item) = item_model::Entity::find_safety_by_id(id)
            .one(&DB_CONN.wait().pg_conn)
            .await?
        {
            let mut am: item_model::ActiveModel = item.into();
            // 复制为新行：IDENTITY 列走自增（显式 Set(0) 会在第二次复制时撞主键）
            am.id = NotSet;
            am.area_id = Set(area_id);
            am.create_time = Set(chrono::Utc::now().naive_utc());
            am.update_time = Set(None);
            let res = am.insert(&DB_CONN.wait().pg_conn).await?;
            let new_id = res.id;
            // 复制类型关联
            let links = link_model::Entity::find_safety()
                .filter(link_model::Column::ItemId.eq(id))
                .all(&DB_CONN.wait().pg_conn)
                .await?;
            for l in links {
                let active = link_model::ActiveModel {
                    version: Set(0),
                    id: NotSet,
                    create_time: Set(chrono::Utc::now().naive_utc()),
                    update_time: Set(None),
                    creator_id: Set(None),
                    updater_id: Set(None),
                    del_flag: Set(false),

                    type_id: Set(l.type_id),
                    item_id: Set(new_id),
                };
                active.insert(&DB_CONN.wait().pg_conn).await?;
            }
            count += 1;
        }
    }
    Ok(CommonResponse::new(Ok(CopyCountResponse { count })))
}

pub async fn do_add(
    auth: AuthInfo,
    payload: ItemAddRequest,
) -> Result<CommonResponse<ItemAddResponse>> {
    auth.require_non_anonymous()?;
    let now = chrono::Utc::now().naive_utc();

    let active = item_model::ActiveModel {
        version: Set(0),
        id: NotSet,
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),

        name: Set(payload.name),
        area_id: Set(payload.area_id),
        default_refresh_time: Set(payload.default_refresh_time.unwrap_or(0)),
        default_content: Set(Some(payload.default_content)),
        default_count: Set(payload.default_count as i32),
        icon_id: Set(payload.icon_id),
        icon_style_type: Set(payload.icon_style_type),
        hidden_flag: Set(payload.hidden_flag),
        sort_index: Set(payload.sort_index.unwrap_or(0) as i32),
        special_flag: Set(Some(payload.special_flag as i32)),
    };

    let res = active.insert(&DB_CONN.wait().pg_conn).await?;
    let new_id = res.id;

    // 插入类型关联
    for t in payload.type_id_list {
        let now = chrono::Utc::now().naive_utc();
        let active = link_model::ActiveModel {
            version: Set(0),
            id: NotSet,
            create_time: Set(now),
            update_time: Set(None),
            creator_id: Set(None),
            updater_id: Set(None),
            del_flag: Set(false),

            type_id: Set(t),
            item_id: Set(new_id),
        };
        active.insert(&DB_CONN.wait().pg_conn).await?;
    }

    Ok(CommonResponse::new(Ok(ItemAddResponse { id: new_id })))
}
