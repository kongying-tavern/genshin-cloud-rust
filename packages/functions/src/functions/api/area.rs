use anyhow::{Result, anyhow};

use sea_orm::{
    ActiveValue::{NotSet, Set},
    prelude::*,
};

use _database::{DB_CONN, models::area::area as area_model};
use _utils::models::common::EmptyResponse;
use _utils::{
    db_operations::SafeEntityTrait,
    jwt::AuthInfo,
    models::{
        AreaAddRequest, AreaListRequest, AreaListResponse, AreaUpdateRequest, AreaVO,
        wrapper::CommonResponse,
    },
};

// 新增地区
pub async fn do_add(auth: AuthInfo, payload: AreaAddRequest) -> Result<CommonResponse<i64>> {
    auth.require_non_anonymous()?;
    let now = chrono::Utc::now().naive_utc();

    let icon_id = resolve_icon_id(payload.icon_id, payload.icon_tag.as_deref()).await?;

    let active = area_model::ActiveModel {
        version: Set(0),
        id: NotSet,
        create_time: Set(now),
        update_time: Set(None),
        creator_id: Set(Some(auth.info.id)),
        updater_id: Set(None),
        del_flag: Set(false),

        name: Set(payload.name),
        code: Set(payload.code),
        content: Set(payload.content),
        icon_id: Set(icon_id),
        parent_id: Set(payload.parent_id),
        is_final: Set(payload.is_final),
        hidden_flag: Set(payload.hidden_flag),
        sort_index: Set(payload.sort_index),
        special_flag: Set(payload.special_flag),
    };

    let res = active.insert(&DB_CONN.wait().pg_conn).await?;
    Ok(CommonResponse::new(Ok(res.id)))
}

// 更新地区
pub async fn do_update(
    auth: AuthInfo,
    payload: AreaUpdateRequest,
) -> Result<CommonResponse<EmptyResponse>> {
    auth.require_non_anonymous()?;
    let item = area_model::Entity::find_safety_by_id(payload.id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow!("Area not found"))?;

    let mut am: area_model::ActiveModel = item.into();
    am.name = Set(payload.area.name);
    am.code = Set(payload.area.code);
    am.content = Set(payload.area.content);
    am.icon_id =
        Set(resolve_icon_id(payload.area.icon_id, payload.area.icon_tag.as_deref()).await?);
    am.parent_id = Set(payload.area.parent_id);
    am.is_final = Set(payload.area.is_final);
    am.hidden_flag = Set(payload.area.hidden_flag);
    am.sort_index = Set(payload.area.sort_index);
    am.special_flag = Set(payload.area.special_flag);

    area_model::Entity::update_safety(am)?
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
    // Java 契约：parentId <= 0（如 -1）表示“不按父级过滤、查全部”，
    // 前端 area store 固定传 { parentId: -1, isTraverse: true }。
    if let Some(parent) = payload.parent_id.filter(|p| *p > 0) {
        query = query.filter(area_model::Column::ParentId.eq(parent));
    }
    if let Some(hidden_flag) = payload.hidden_flag {
        // 数据级过滤：与 marker 域的 hiddenFlagList 过滤保持一致，
        // 让客户端按 normal / insider 数据级请求地区列表。
        query = query.filter(area_model::Column::HiddenFlag.eq(hidden_flag));
    }

    let icon_tag_map = super::icon::icon_tag_map(&DB_CONN.wait().pg_conn).await?;
    let items = query.all(&DB_CONN.wait().pg_conn).await?;
    let mut ret = Vec::with_capacity(items.len());
    for it in items {
        ret.push(AreaVO {
            version: it.version,
            id: it.id,
            create_time: it.create_time.and_utc().timestamp_millis() as f64,
            update_time: it
                .update_time
                .map(|dt| dt.and_utc().timestamp_millis() as f64),
            creator_id: it.creator_id,
            updater_id: it.updater_id,
            name: it.name,
            code: it.code,
            content: it.content,
            icon_tag: Some(icon_tag_map.get(&it.icon_id).cloned().unwrap_or_default()),
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
pub async fn do_get(_auth: AuthInfo, area_id: i64) -> Result<CommonResponse<AreaVO>> {
    let icon_tag_map = super::icon::icon_tag_map(&DB_CONN.wait().pg_conn).await?;
    let item = area_model::Entity::find_safety_by_id(area_id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow!("Area not found"))?;
    Ok(CommonResponse::new(Ok(AreaVO {
        version: item.version,
        id: item.id,
        create_time: item.create_time.and_utc().timestamp_millis() as f64,
        update_time: item
            .update_time
            .map(|dt| dt.and_utc().timestamp_millis() as f64),
        creator_id: item.creator_id,
        updater_id: item.updater_id,
        name: item.name,
        code: item.code,
        content: item.content,
        icon_tag: Some(icon_tag_map.get(&item.icon_id).cloned().unwrap_or_default()),
        icon_id: item.icon_id,
        parent_id: item.parent_id,
        is_final: item.is_final,
        hidden_flag: item.hidden_flag,
        sort_index: item.sort_index,
        special_flag: item.special_flag,
    })))
}

// 删除（软删除）
pub async fn do_delete(auth: AuthInfo, area_id: i64) -> Result<CommonResponse<EmptyResponse>> {
    auth.require_non_anonymous()?;
    let item = area_model::Entity::find_safety_by_id(area_id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow!("Area not found"))?;
    let mut am: area_model::ActiveModel = item.into();
    am.del_flag = Set(true);
    area_model::Entity::delete_safety(am)?
        .exec(&DB_CONN.wait().pg_conn)
        .await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
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
