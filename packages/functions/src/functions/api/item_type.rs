use anyhow::{anyhow, Result};

use sea_orm::{prelude::*, ActiveValue::Set, QueryFilter, QuerySelect};

use _database::{
    models::item::item_type as item_type_model, models::item::item_type_link as link_model, DB_CONN,
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

// 更新类型
pub async fn do_update(
    _auth: AuthInfo,
    payload: ItemTypeUpdateData,
) -> Result<CommonResponse<EmptyResponse>> {
    let item = item_type_model::Entity::find_safety_by_id(payload.id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow!("ItemType not found"))?;

    let mut am: item_type_model::ActiveModel = item.into();
    // icon_tag -> icon_id
    let icon_id_val = payload.icon_tag.parse::<i64>().unwrap_or(0);
    am.icon_id = Set(icon_id_val);

    if let Some(name) = payload.name {
        am.name = Set(name);
    }
    am.content = Set(payload.content);
    am.parent_id = Set(payload.parent_id);
    am.is_final = Set(payload.is_final);
    am.hidden_flag = Set(payload.hidden_flag);
    if let Some(si) = payload.sort_index {
        am.sort_index = Set(si as i32);
    }

    item_type_model::Entity::update_safety(am)
        .exec(&DB_CONN.wait().pg_conn)
        .await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

// 将一组 item_id 移动到目标类型（更新 link 表的 type_id）
pub async fn do_move_to_target(
    _auth: AuthInfo,
    target_type_id: i64,
    payload: Vec<i64>,
) -> Result<CommonResponse<EmptyResponse>> {
    for item_id in payload {
        // 查找 link 记录（末端类型）
        let links = link_model::Entity::find_safety()
            .filter(link_model::Column::ItemId.eq(item_id))
            .all(&DB_CONN.wait().pg_conn)
            .await?;

        for link in links {
            let mut lam: link_model::ActiveModel = link.into();
            lam.type_id = Set(target_type_id);
            link_model::Entity::update_safety(lam)
                .exec(&DB_CONN.wait().pg_conn)
                .await?;
        }
    }
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

// 列表（分页/筛选）
pub async fn do_get_list(
    _auth: AuthInfo,
    self_flag: bool,
    payload: ItemTypeListRequest,
) -> Result<CommonResponse<ItemTypeListResponse>> {
    let _ = self_flag; // reserved for permission filtering

    let db = &DB_CONN.wait().pg_conn;
    let mut query = item_type_model::Entity::find_safety();
    if let Some(type_list) = payload.type_id_list {
        query = query.filter(item_type_model::Column::Id.is_in(type_list));
    }

    let size = payload.page.size.unwrap_or(10) as u64;
    let current = payload.page.current.unwrap_or(1);
    let offset = (current.saturating_sub(1) as u64).saturating_mul(size);

    let total = query.clone().count(db).await?;
    let items = query.limit(size).offset(offset).all(db).await?;

    let items_val: Vec<ItemTypeVO> = items
        .into_iter()
        .map(|i| ItemTypeVO {
            id: i.id,
            name: i.name,
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
    };
    Ok(CommonResponse::new(Ok(body)))
}

pub async fn do_get_list_all(_auth: AuthInfo) -> Result<CommonResponse<ItemTypeAllResponse>> {
    let items = item_type_model::Entity::find_safety()
        .all(&DB_CONN.wait().pg_conn)
        .await?;
    let vec = items
        .into_iter()
        .map(|i| ItemTypeVO {
            id: i.id,
            name: i.name,
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
pub async fn do_delete(_auth: AuthInfo, id: i64) -> Result<CommonResponse<EmptyResponse>> {
    let item = item_type_model::Entity::find_safety_by_id(id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow!("ItemType not found"))?;
    let mut am: item_type_model::ActiveModel = item.into();
    am.del_flag = Set(true);
    item_type_model::Entity::delete_safety(am)
        .exec(&DB_CONN.wait().pg_conn)
        .await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

// 新增类型
pub async fn do_add(_auth: AuthInfo, payload: ItemTypeAddRequest) -> Result<i64> {
    let now = chrono::Utc::now().naive_utc();
    // name 在逻辑上为必填
    let name = payload.name.ok_or(anyhow!("name required"))?;
    let icon_id_val = payload.icon_tag.parse::<i64>().unwrap_or(0);
    let sort_index = payload.sort_index.unwrap_or(0) as i32;

    let active = item_type_model::ActiveModel {
        version: Set(0),
        id: Set(0),
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),

        icon_id: Set(icon_id_val),
        name: Set(name),
        content: Set(payload.content),
        parent_id: Set(payload.parent_id),
        is_final: Set(payload.is_final),
        hidden_flag: Set(payload.hidden_flag),
        sort_index: Set(sort_index),
    };

    let res = active.insert(&DB_CONN.wait().pg_conn).await?;
    Ok(res.id)
}
