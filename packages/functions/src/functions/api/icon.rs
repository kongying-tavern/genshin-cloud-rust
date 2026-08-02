use anyhow::{Result, anyhow};

use chrono::Utc;

use sea_orm::{
    ActiveValue::{NotSet, Set},
    QuerySelect,
    prelude::*,
};

use _database::DB_CONN;
use _database::models::icon::icon as icon_model;
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        IconAddRequest, IconListRequest, IconUpdateRequest,
        icon::{IconAddResponse, IconListResponse, IconSingleResponse, IconVO},
        wrapper::CommonResponse,
    },
};

// 新增图标
pub async fn do_add(
    auth: AuthInfo,
    payload: IconAddRequest,
) -> Result<CommonResponse<IconAddResponse>> {
    auth.require_non_anonymous()?;
    let now = Utc::now().naive_utc();

    let active = icon_model::ActiveModel {
        version: Set(0),
        id: NotSet,
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(Some(auth.info.id)),
        updater_id: Set(None),
        del_flag: Set(false),

        name: Set(payload.name),
        url: Set(payload.url),
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
    if let Some(ids) = payload.icon_list
        && !ids.is_empty()
    {
        query = query.filter(icon_model::Column::Id.is_in(ids));
    }
    if let Some(name) = payload.name {
        query = query.filter(icon_model::Column::Name.contains(name));
    }

    let total = query.clone().count(&DB_CONN.wait().pg_conn).await?;

    let mut select = query;
    if let Some(current) = payload.page.current
        && let Some(size) = payload.page.size
    {
        let offset = (current.saturating_sub(1) as u64).saturating_mul(size as u64);
        select = select.limit(size as u64).offset(offset);
    }

    let items = select.all(&DB_CONN.wait().pg_conn).await?;
    let mut arr = Vec::with_capacity(items.len());
    for it in items {
        arr.push(IconVO {
            id: it.id,
            name: it.name,
            url: it.url,
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
            name: item.name,
            url: item.url,
        },
    };
    Ok(CommonResponse::new(Ok(payload)))
}

// 删除（软删除）
pub async fn do_delete(auth: AuthInfo, id: i64) -> Result<CommonResponse<()>> {
    auth.require_non_anonymous()?;
    let item = icon_model::Entity::find_safety_by_id(id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow!("Icon not found"))?;
    let mut am: icon_model::ActiveModel = item.into();
    am.del_flag = Set(true);
    icon_model::Entity::delete_safety(am)?
        .exec(&DB_CONN.wait().pg_conn)
        .await?;
    Ok(CommonResponse::new(Ok(())))
}

// 更新图标
pub async fn do_update(auth: AuthInfo, payload: IconUpdateRequest) -> Result<CommonResponse<()>> {
    auth.require_non_anonymous()?;
    let item = icon_model::Entity::find_safety_by_id(payload.id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow!("Icon not found"))?;
    let mut am: icon_model::ActiveModel = item.into();
    am.name = Set(payload.base.name);
    am.url = Set(payload.base.url);
    icon_model::Entity::update_safety(am)?
        .exec(&DB_CONN.wait().pg_conn)
        .await?;
    Ok(CommonResponse::new(Ok(())))
}
