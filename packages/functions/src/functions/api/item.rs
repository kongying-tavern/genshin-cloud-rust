use anyhow::{anyhow, Result};

// serde_json not needed after concrete response conversion

use sea_orm::{prelude::*, ActiveValue::Set, QueryFilter, QuerySelect};

use _database::{
    models::item::item as item_model, models::item::item_type_link as link_model, DB_CONN,
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

// 批量更新物品（支持单条或多条）
pub async fn do_update(
    _auth: AuthInfo,
    _edit_same: bool,
    payload: Vec<ItemUpdateData>,
) -> Result<CommonResponse<()>> {
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
        am.icon_id = Set(p.icon_tag.parse::<i64>().unwrap_or(0));
        am.icon_style_type = Set(p.icon_style_type);
        am.hidden_flag = Set(p.hidden_flag);
        if let Some(si) = p.sort_index {
            am.sort_index = Set(si as i32);
        }
        am.special_flag = Set(Some(p.special_flag as i32));

        item_model::Entity::update_safety(am)
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
    let items = query.limit(size).offset(offset).all(db).await?;
    let mut arr = Vec::with_capacity(items.len());
    for it in items {
        arr.push(ItemVO {
            id: it.id,
            name: it.name,
            area_id: it.area_id,
            default_refresh_time: it.default_refresh_time,
            default_content: it.default_content,
            default_count: it.default_count,
            icon_id: it.icon_id,
            icon_style_type: it.icon_style_type,
            hidden_flag: it.hidden_flag,
            sort_index: it.sort_index,
            special_flag: it.special_flag,
        });
    }
    let payload = ItemListResponse {
        total: total as i64,
        items: arr,
    };
    Ok(CommonResponse::new(Ok(payload)))
}

// 将多个物品加入到某个类型（在 link 表中插入或更新）
pub async fn do_join_type(
    _auth: AuthInfo,
    type_id: i64,
    payload: Vec<i64>,
) -> Result<CommonResponse<()>> {
    for item_id in payload {
        // 查找现有 link
        let ex = link_model::Entity::find_safety()
            .filter(link_model::Column::ItemId.eq(item_id))
            .one(&DB_CONN.wait().pg_conn)
            .await?;
        if let Some(link) = ex {
            let mut lam: link_model::ActiveModel = link.into();
            lam.type_id = Set(type_id);
            link_model::Entity::update_safety(lam)
                .exec(&DB_CONN.wait().pg_conn)
                .await?;
        } else {
            let now = chrono::Utc::now().naive_utc();
            let active = link_model::ActiveModel {
                version: Set(0),
                id: Set(0),
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
    let db = &DB_CONN.wait().pg_conn;
    let items = item_model::Entity::find_safety()
        .filter(item_model::Column::Id.is_in(payload))
        .all(db)
        .await?;
    let mut arr = Vec::with_capacity(items.len());
    for it in items {
        arr.push(ItemVO {
            id: it.id,
            name: it.name,
            area_id: it.area_id,
            default_refresh_time: it.default_refresh_time,
            default_content: it.default_content,
            default_count: it.default_count,
            icon_id: it.icon_id,
            icon_style_type: it.icon_style_type,
            hidden_flag: it.hidden_flag,
            sort_index: it.sort_index,
            special_flag: it.special_flag,
        });
    }
    let payload = ItemListResponse {
        total: arr.len() as i64,
        items: arr,
    };
    Ok(CommonResponse::new(Ok(payload)))
}

pub async fn do_delete(_auth: AuthInfo, id: i64) -> Result<CommonResponse<()>> {
    let item = item_model::Entity::find_safety_by_id(id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow!("Item not found"))?;
    let mut am: item_model::ActiveModel = item.into();
    am.del_flag = Set(true);
    item_model::Entity::delete_safety(am)
        .exec(&DB_CONN.wait().pg_conn)
        .await?;
    Ok(CommonResponse::new(Ok(())))
}

// 复制物品到指定地区（简单实现：复制记录并关联相同类型）
pub async fn do_copy_to_area(
    _auth: AuthInfo,
    area_id: i64,
    payload: Vec<i64>,
) -> Result<CommonResponse<CopyCountResponse>> {
    let mut count = 0i64;
    for id in payload {
        if let Some(item) = item_model::Entity::find_safety_by_id(id)
            .one(&DB_CONN.wait().pg_conn)
            .await?
        {
            let mut am: item_model::ActiveModel = item.into();
            am.id = Set(0);
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
                    id: Set(0),
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
    _auth: AuthInfo,
    payload: ItemAddRequest,
) -> Result<CommonResponse<ItemAddResponse>> {
    let now = chrono::Utc::now().naive_utc();
    let icon_id_val = payload.icon_tag.parse::<i64>().unwrap_or(0);

    let active = item_model::ActiveModel {
        version: Set(0),
        id: Set(0),
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
        icon_id: Set(icon_id_val),
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
            id: Set(0),
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
