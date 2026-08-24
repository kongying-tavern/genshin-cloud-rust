use anyhow::{Result, anyhow};

use sea_orm::{
    ActiveValue::{NotSet, Set},
    QuerySelect,
    prelude::*,
};

use _database::{
    DB_CONN,
    models::{
        area::area as area_model, item::item as item_model, marker::marker as marker_model,
        marker::marker_item_link as mil_model,
    },
};
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
    let db = &DB_CONN.wait().pg_conn;
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
        // Java createArea：新地区恒为末端（客户端传入的 isFinal 被忽略）。
        is_final: Set(true),
        hidden_flag: Set(payload.hidden_flag),
        sort_index: Set(payload.sort_index),
        special_flag: Set(payload.special_flag),
    };

    let res = active.insert(db).await?;
    // Java createArea：父级（>0）因新增子级不再是末端。
    set_parent_is_final(db, payload.parent_id, false).await?;
    Ok(CommonResponse::new(Ok(res.id)))
}

// 更新地区
pub async fn do_update(
    auth: AuthInfo,
    payload: AreaUpdateRequest,
) -> Result<CommonResponse<EmptyResponse>> {
    auth.require_non_anonymous()?;
    // Java updateArea：指定的父节点无效（自己挂到自己下面）。
    if payload.area.parent_id == payload.id {
        return Err(anyhow!("指定的父节点无效"));
    }
    let db = &DB_CONN.wait().pg_conn;
    let item = area_model::Entity::find_safety_by_id(payload.id)
        .one(db)
        .await?;
    let item = item.ok_or(anyhow!("Area not found"))?;

    // Java updateArea：父级变化时，新父级不再是末端，旧父级按剩余子级重算。
    let old_parent_id = item.parent_id;
    if old_parent_id != payload.area.parent_id {
        set_parent_is_final(db, payload.area.parent_id, false).await?;
        recalc_is_final(db, old_parent_id).await?;
    }

    let mut am: area_model::ActiveModel = item.into();
    am.name = Set(payload.area.name);
    am.code = Set(payload.area.code);
    am.content = Set(payload.area.content);
    am.icon_id =
        Set(resolve_icon_id(payload.area.icon_id, payload.area.icon_tag.as_deref()).await?);
    am.parent_id = Set(payload.area.parent_id);
    // Java updateAreaIsFinal(area)：本地区是否末端按子级数量重算，
    // 客户端传入的 isFinal 被忽略。
    am.is_final = Set(is_leaf(db, payload.id).await?);
    am.hidden_flag = Set(payload.area.hidden_flag);
    am.sort_index = Set(payload.area.sort_index);
    am.special_flag = Set(payload.area.special_flag);

    area_model::Entity::update_safety(am)?.exec(db).await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

/// 该地区是否为末端（无非删除子级）。Java updateAreaIsFinal(area) 的判定。
async fn is_leaf(db: &sea_orm::DatabaseConnection, area_id: i64) -> Result<bool> {
    let children = area_model::Entity::find_safety()
        .filter(area_model::Column::ParentId.eq(area_id))
        .count(db)
        .await?;
    Ok(children == 0)
}

/// Java updateAreaIsFinal(parentId, isFinal)：直接设置地区（父级）的末端标志。
/// parentId <= 0（如根级 -1）表示无父级，跳过。
async fn set_parent_is_final(
    db: &sea_orm::DatabaseConnection,
    parent_id: i64,
    is_final: bool,
) -> Result<()> {
    if parent_id <= 0 {
        return Ok(());
    }
    area_model::Entity::update_many()
        .col_expr(area_model::Column::IsFinal, Expr::value(is_final))
        .filter(area_model::Column::Id.eq(parent_id))
        .exec(db)
        .await?;
    Ok(())
}

/// Java recalculateAreaIsFinal：按剩余非删除子级数量重算末端标志。
async fn recalc_is_final(db: &sea_orm::DatabaseConnection, area_id: i64) -> Result<()> {
    set_parent_is_final(db, area_id, is_leaf(db, area_id).await?).await
}

// 列表
pub async fn do_list(
    auth: AuthInfo,
    payload: AreaListRequest,
) -> Result<CommonResponse<AreaListResponse>> {
    // 可见性（Java getAreaList 的 hiddenFlagList）：按调用者角色过滤，
    // 无此过滤时隐藏/测试服地区会泄露给普通用户。
    let allowed = _utils::types::allowed_hidden_flags(auth.info.role_id);
    let mut query =
        area_model::Entity::find_safety().filter(area_model::Column::HiddenFlag.is_in(allowed));
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
pub async fn do_get(auth: AuthInfo, area_id: i64) -> Result<CommonResponse<AreaVO>> {
    let icon_tag_map = super::icon::icon_tag_map(&DB_CONN.wait().pg_conn).await?;
    let item = area_model::Entity::find_safety_by_id(area_id)
        .one(&DB_CONN.wait().pg_conn)
        .await?;
    let item = item.ok_or(anyhow!("Area not found"))?;
    // 可见性：不可见 flag 的地区对调用者如同不存在（Java getArea 的
    // hiddenFlagList 过滤同口径）。
    let allowed = _utils::types::allowed_hidden_flags(auth.info.role_id);
    if !allowed.contains(&(item.hidden_flag as i32)) {
        return Err(anyhow!("Area not found"));
    }
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

// 删除（软删除，递归子树并级联物品/点位，Java deleteArea 同语义）
pub async fn do_delete(auth: AuthInfo, area_id: i64) -> Result<CommonResponse<EmptyResponse>> {
    auth.require_non_anonymous()?;
    let db = &DB_CONN.wait().pg_conn;
    let item = area_model::Entity::find_safety_by_id(area_id)
        .one(db)
        .await?;
    let item = item.ok_or(anyhow!("Area not found"))?;
    let parent_area_id = item.parent_id;

    // 逐层删除子树：先删本层地区与其中物品/点位，再找下一层子地区。
    let mut now_ids: Vec<i64> = vec![area_id];
    while !now_ids.is_empty() {
        // 软删本层地区
        area_model::Entity::update_many()
            .col_expr(area_model::Column::DelFlag, Expr::value(true))
            .filter(area_model::Column::Id.is_in(&now_ids))
            .exec(db)
            .await?;
        delete_marker_and_item_in_area(db, &now_ids).await?;
        // 下一层：刚被软删地区的子级（del_flag 过滤由 find_safety 施加，
        // 因此必须在软删前把子级查出来——改为先查后删的顺序见下）。
        now_ids = area_model::Entity::find_safety()
            .filter(area_model::Column::ParentId.is_in(&now_ids))
            .select_only()
            .column(area_model::Column::Id)
            .into_tuple::<i64>()
            .all(db)
            .await?;
    }

    // Java deleteArea 收尾：重算父级 isFinal（无剩余子级 → 叶子）
    recalc_is_final(db, parent_area_id).await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}

/// Java deleteMarkerAndItemInArea 同语义：删除地区内的物品，以及只关联到
/// 这些物品的点位（仍关联其他地区物品的点位保留）。与 Java 不同的是删除
/// 物品时按 itemIdList（Java 原样复刻会把 areaIdList 当 itemIdList 用）。
async fn delete_marker_and_item_in_area(
    db: &sea_orm::DatabaseConnection,
    area_ids: &[i64],
) -> Result<()> {
    let item_ids: Vec<i64> = item_model::Entity::find_safety()
        .filter(item_model::Column::AreaId.is_in(area_ids.to_vec()))
        .select_only()
        .column(item_model::Column::Id)
        .into_tuple::<i64>()
        .all(db)
        .await?;
    if item_ids.is_empty() {
        return Ok(());
    }
    // 软删物品（连同其类型关联）
    item_model::Entity::update_many()
        .col_expr(item_model::Column::DelFlag, Expr::value(true))
        .filter(item_model::Column::Id.is_in(&item_ids))
        .exec(db)
        .await?;

    // 受影响的点位
    let marker_ids: Vec<i64> = mil_model::Entity::find_safety()
        .filter(mil_model::Column::ItemId.is_in(&item_ids))
        .select_only()
        .column(mil_model::Column::MarkerId)
        .into_tuple::<i64>()
        .all(db)
        .await?;
    if marker_ids.is_empty() {
        return Ok(());
    }
    // 删除这些物品的点位-物品关联
    mil_model::Entity::update_many()
        .col_expr(mil_model::Column::DelFlag, Expr::value(true))
        .filter(mil_model::Column::ItemId.is_in(&item_ids))
        .exec(db)
        .await?;
    // 仍关联其他（未删除）物品的点位保留，其余软删
    let surviving: Vec<i64> = mil_model::Entity::find_safety()
        .filter(mil_model::Column::MarkerId.is_in(&marker_ids))
        .select_only()
        .column(mil_model::Column::MarkerId)
        .into_tuple::<i64>()
        .all(db)
        .await?;
    let surviving: std::collections::HashSet<i64> = surviving.into_iter().collect();
    let to_delete: Vec<i64> = marker_ids
        .iter()
        .copied()
        .filter(|mid| !surviving.contains(mid))
        .collect();
    if !to_delete.is_empty() {
        marker_model::Entity::update_many()
            .col_expr(marker_model::Column::DelFlag, Expr::value(true))
            .filter(marker_model::Column::Id.is_in(&to_delete))
            .exec(db)
            .await?;
    }
    Ok(())
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
