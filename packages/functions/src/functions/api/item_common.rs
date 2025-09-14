use anyhow::{anyhow, Result};
use chrono::Utc;

use sea_orm::{prelude::*, ActiveValue::Set, QuerySelect};

use _utils::{
    jwt::AuthInfo,
    models::{
        common::EmptyResponse,
        item::{ItemAddResponse, ItemListResponse, ItemSingleResponse, ItemVO},
        wrapper::CommonResponse,
        wrapper::Pagination,
    },
    types::HiddenFlag,
};

use _utils::db_operations::SafeEntityTrait;

use _database::{models::item::item as item_model, DB_CONN};

pub async fn do_get_list(
    _auth: AuthInfo,
    payload: Pagination,
) -> Result<CommonResponse<ItemListResponse>> {
    let db = &DB_CONN.wait().pg_conn;

    let size = payload.size.unwrap_or(10) as u64;
    let current = payload.current.unwrap_or(1);
    let offset = (current.saturating_sub(1) as u64).saturating_mul(size);

    let total = item_model::Entity::find_safety().clone().count(db).await?;
    let items = item_model::Entity::find_safety()
        .limit(size)
        .offset(offset)
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
        total: total as i64,
        items: arr,
    };
    Ok(CommonResponse::new(Ok(payload)))
}

pub async fn do_get_single(_auth: AuthInfo, id: i64) -> Result<CommonResponse<ItemSingleResponse>> {
    let item = item_model::Entity::find_safety_by_id(id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow!("Item not found"))?;
    let payload = ItemSingleResponse {
        item: ItemVO {
            id: item.id,
            name: item.name,
            area_id: item.area_id,
            default_refresh_time: item.default_refresh_time,
            default_content: item.default_content,
            default_count: item.default_count,
            icon_id: item.icon_id,
            icon_style_type: item.icon_style_type,
            hidden_flag: item.hidden_flag,
            sort_index: item.sort_index,
            special_flag: item.special_flag,
        },
    };
    Ok(CommonResponse::new(Ok(payload)))
}

pub async fn do_add(_auth: AuthInfo, payload: Vec<i64>) -> Result<CommonResponse<ItemAddResponse>> {
    let now = Utc::now().naive_utc();

    let active = item_model::ActiveModel {
        version: Set(0),
        id: Set(0),
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(None),
        updater_id: Set(None),
        del_flag: Set(false),

        name: Set("新物品".to_string()),
        area_id: Set(payload.get(0).cloned().unwrap_or(0)),
        default_refresh_time: Set(payload.get(2).cloned().unwrap_or(0) as i64),
        default_content: Set(None),
        default_count: Set(payload.get(1).cloned().unwrap_or(1) as i32),
        icon_id: Set(0),
        icon_style_type: Set(_utils::types::IconStyleType::Default),
        hidden_flag: Set(HiddenFlag::Visible),
        sort_index: Set(0),
        special_flag: Set(None),
        ..Default::default()
    };

    let res = active.insert(&DB_CONN.wait().pg_conn).await?;
    Ok(CommonResponse::new(Ok(ItemAddResponse { id: res.id })))
}

pub async fn do_update(
    _auth: AuthInfo,
    payload: serde_json::Value,
) -> Result<CommonResponse<EmptyResponse>> {
    let id = payload.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    let item = item_model::Entity::find_safety_by_id(id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow::anyhow!("Item not found"))?;

    let mut am: item_model::ActiveModel = item.into();
    if let Some(name) = payload.get("name").and_then(|v| v.as_str()) {
        am.name = Set(name.to_string());
    }
    if let Some(default_content) = payload.get("defaultContent").and_then(|v| v.as_str()) {
        am.default_content = Set(Some(default_content.to_string()));
    }

    item_model::Entity::update_safety(am)
        .exec(&DB_CONN.wait().pg_conn)
        .await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

pub async fn do_delete(_auth: AuthInfo, id: i64) -> Result<CommonResponse<EmptyResponse>> {
    let item = item_model::Entity::find_safety_by_id(id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow::anyhow!("Item not found"))?;
    let mut am: item_model::ActiveModel = item.into();
    am.del_flag = Set(true);
    item_model::Entity::delete_safety(am)
        .exec(&DB_CONN.wait().pg_conn)
        .await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}
