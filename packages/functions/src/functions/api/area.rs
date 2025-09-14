use anyhow::{anyhow, Result};

use sea_orm::{prelude::*, ActiveValue::Set};

use _database::{models::area::area as area_model, DB_CONN};
use _utils::models::common::EmptyResponse;
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        wrapper::CommonResponse, AreaAddRequest, AreaAddResponse, AreaListRequest,
        AreaListResponse, AreaSingleResponse, AreaUpdateRequest, AreaVO,
    },
};

// 新增地区
pub async fn do_add(
    auth: AuthInfo,
    payload: AreaAddRequest,
) -> Result<CommonResponse<AreaAddResponse>> {
    let now = chrono::Utc::now().naive_utc();

    let icon_id_val = payload.icon_tag.parse::<i64>().unwrap_or(0);

    let active = area_model::ActiveModel {
        version: Set(0),
        id: Set(0),
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(Some(auth.info.id)),
        updater_id: Set(None),
        del_flag: Set(false),

        name: Set(payload.name),
        code: Set(payload.code),
        content: Set(payload.content),
        icon_id: Set(icon_id_val),
        parent_id: Set(payload.parent_id),
        is_final: Set(payload.is_final),
        hidden_flag: Set(payload.hidden_flag),
        sort_index: Set(payload.sort_index),
        special_flag: Set(payload.special_flag),
    };

    let res = active.insert(&DB_CONN.wait().pg_conn).await?;
    Ok(CommonResponse::new(Ok(AreaAddResponse { id: res.id })))
}

// 更新地区
pub async fn do_update(
    _auth: AuthInfo,
    payload: AreaUpdateRequest,
) -> Result<CommonResponse<EmptyResponse>> {
    let item = area_model::Entity::find_safety_by_id(payload.id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow!("Area not found"))?;

    let mut am: area_model::ActiveModel = item.into();
    am.name = Set(payload.area.name);
    am.code = Set(payload.area.code);
    am.content = Set(payload.area.content);
    am.icon_id = Set(payload.area.icon_tag.parse::<i64>().unwrap_or(0));
    am.parent_id = Set(payload.area.parent_id);
    am.is_final = Set(payload.area.is_final);
    am.hidden_flag = Set(payload.area.hidden_flag);
    am.sort_index = Set(payload.area.sort_index);
    am.special_flag = Set(payload.area.special_flag);

    area_model::Entity::update_safety(am)
        .exec(&DB_CONN.wait().pg_conn)
        .await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

// 列表
pub async fn do_list(
    _auth: AuthInfo,
    payload: AreaListRequest,
) -> Result<CommonResponse<AreaListResponse>> {
    let mut query = area_model::Entity::find_safety();
    if let Some(parent) = payload.parent_id {
        query = query.filter(area_model::Column::ParentId.eq(parent));
    }

    let items = query.all(&DB_CONN.wait().pg_conn).await?;
    let mut ret = Vec::with_capacity(items.len());
    for it in items {
        ret.push(AreaVO {
            id: it.id,
            name: it.name,
            code: it.code,
            content: it.content,
            icon_id: it.icon_id,
            parent_id: it.parent_id,
            is_final: it.is_final,
            hidden_flag: it.hidden_flag,
            sort_index: it.sort_index,
            special_flag: it.special_flag,
        });
    }
    Ok(CommonResponse::new(Ok(AreaListResponse(ret))))
}

// 获取单个
pub async fn do_get(_auth: AuthInfo, area_id: i64) -> Result<CommonResponse<AreaSingleResponse>> {
    let item = area_model::Entity::find_safety_by_id(area_id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow!("Area not found"))?;
    Ok(CommonResponse::new(Ok(AreaSingleResponse {
        item: AreaVO {
            id: item.id,
            name: item.name,
            code: item.code,
            content: item.content,
            icon_id: item.icon_id,
            parent_id: item.parent_id,
            is_final: item.is_final,
            hidden_flag: item.hidden_flag,
            sort_index: item.sort_index,
            special_flag: item.special_flag,
        },
    })))
}

// 删除（软删除）
pub async fn do_delete(_auth: AuthInfo, area_id: i64) -> Result<CommonResponse<EmptyResponse>> {
    let item = area_model::Entity::find_safety_by_id(area_id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow!("Area not found"))?;
    let mut am: area_model::ActiveModel = item.into();
    am.del_flag = Set(true);
    area_model::Entity::delete_safety(am)
        .exec(&DB_CONN.wait().pg_conn)
        .await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}
