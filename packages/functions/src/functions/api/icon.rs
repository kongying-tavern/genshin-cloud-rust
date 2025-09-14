use anyhow::{anyhow, Result};

use chrono::Utc;

use sea_orm::{prelude::*, ActiveValue::Set, QuerySelect};

use _database::models::icon::icon as icon_model;
use _database::DB_CONN;
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        icon::{IconAddResponse, IconListResponse, IconSingleResponse, IconVO},
        wrapper::CommonResponse,
        IconAddRequest, IconListRequest, IconUpdateRequest,
    },
};

// 新增图标
pub async fn do_add(
    auth: AuthInfo,
    payload: IconAddRequest,
) -> Result<CommonResponse<IconAddResponse>> {
    let now = Utc::now().naive_utc();

    let active = icon_model::ActiveModel {
        version: Set(0),
        id: Set(0),
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(Some(auth.info.id)),
        updater_id: Set(None),
        del_flag: Set(false),

        // 数据库列：url, tag, description, url_variants
        url: Set(payload.url),
        tag: Set(payload.name),
        description: Set(String::new()),
        url_variants: Set(Default::default()),
    };

    let res = active.insert(&DB_CONN.wait().pg_conn).await?;
    Ok(CommonResponse::new(Ok(IconAddResponse { id: res.id })))
}

// 列表查询（支持分页）
pub async fn do_list(
    _auth: AuthInfo,
    payload: IconListRequest,
) -> Result<CommonResponse<IconListResponse>> {
    let mut query = icon_model::Entity::find_safety();
    if let Some(creator) = payload.creator {
        query = query.filter(icon_model::Column::CreatorId.eq(creator));
    }
    if let Some(ids) = payload.icon_list {
        if !ids.is_empty() {
            query = query.filter(icon_model::Column::Id.is_in(ids));
        }
    }
    if let Some(name) = payload.name {
        query = query.filter(icon_model::Column::Tag.contains(name));
    }

    let total = query.clone().count(&DB_CONN.wait().pg_conn).await?;

    let mut select = query;
    if let Some(current) = payload.page.current {
        if let Some(size) = payload.page.size {
            let offset = (current.saturating_sub(1) as u64).saturating_mul(size as u64);
            select = select.limit(size as u64).offset(offset as u64);
        }
    }

    let items = select.all(&DB_CONN.wait().pg_conn).await?;
    let mut arr = Vec::with_capacity(items.len());
    for it in items {
        arr.push(IconVO {
            id: it.id,
            url: it.url,
            tag: it.tag,
            description: it.description,
            url_variants: it.url_variants,
        });
    }
    let payload = IconListResponse {
        total: total as i64,
        items: arr,
    };
    Ok(CommonResponse::new(Ok(payload)))
}

// 获取单个图标
pub async fn do_get_single(_auth: AuthInfo, id: i64) -> Result<CommonResponse<IconSingleResponse>> {
    let item = icon_model::Entity::find_safety_by_id(id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow!("Icon not found"))?;
    let payload = IconSingleResponse {
        item: IconVO {
            id: item.id,
            url: item.url,
            tag: item.tag,
            description: item.description,
            url_variants: item.url_variants,
        },
    };
    Ok(CommonResponse::new(Ok(payload)))
}

// 删除（软删除）
pub async fn do_delete(_auth: AuthInfo, id: i64) -> Result<CommonResponse<()>> {
    let item = icon_model::Entity::find_safety_by_id(id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow!("Icon not found"))?;
    let mut am: icon_model::ActiveModel = item.into();
    am.del_flag = Set(true);
    icon_model::Entity::delete_safety(am)
        .exec(&DB_CONN.wait().pg_conn)
        .await?;
    Ok(CommonResponse::new(Ok(())))
}

// 更新图标
pub async fn do_update(_auth: AuthInfo, payload: IconUpdateRequest) -> Result<CommonResponse<()>> {
    let item = icon_model::Entity::find_safety_by_id(payload.id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow!("Icon not found"))?;
    let mut am: icon_model::ActiveModel = item.into();
    am.tag = Set(payload.base.name);
    am.url = Set(payload.base.url);
    icon_model::Entity::update_safety(am)
        .exec(&DB_CONN.wait().pg_conn)
        .await?;
    Ok(CommonResponse::new(Ok(())))
}
